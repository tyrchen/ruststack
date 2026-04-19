//! A single warm execution instance and its pool.
//!
//! Used by both the native and Docker backends. Each instance owns its own
//! [`runtime_api::RuntimeApiHandle`] and a way to start / kill the
//! corresponding bootstrap (process or container). Pool entries are scoped
//! per `(function_name, qualifier)` because code can diverge between
//! versions.

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tracing::{debug, warn};

use super::{
    error::ExecutorError,
    runtime_api::{self, Job, RuntimeApiHandle, RuntimeResult},
    types::{InvokeRequest, InvokeResponse},
};

/// Identifier for the pool slot — a function/version pair.
pub(crate) type PoolKey = (String, String);

/// Trait that backends implement to create + destroy bootstraps.
///
/// Object-safe so the pool can hold `Arc<dyn InstanceBackend>`.
#[async_trait]
pub(crate) trait InstanceBackend: Send + Sync + std::fmt::Debug {
    /// Spawn a bootstrap pointing at `runtime_api_addr` for the given function.
    /// Returns a handle the pool will keep alive until the instance is reaped.
    async fn spawn(
        &self,
        req: &InvokeRequest,
        runtime_api_addr: std::net::SocketAddr,
    ) -> Result<BackendHandle, ExecutorError>;
}

/// Opaque handle to a backend-specific running thing (process or container).
/// Drop must clean it up; the pool also calls `kill` on graceful shutdown.
pub(crate) trait BackendHandleObj: Send + Sync + std::fmt::Debug {
    fn kill(&mut self);
}

/// Wrapper for object-safety + Drop.
pub(crate) struct BackendHandle(Box<dyn BackendHandleObj>);

impl BackendHandle {
    pub(crate) fn new<H: BackendHandleObj + 'static>(handle: H) -> Self {
        Self(Box::new(handle))
    }

    pub(crate) fn kill(mut self) {
        self.0.kill();
    }
}

impl std::fmt::Debug for BackendHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BackendHandle").field(&self.0).finish()
    }
}

impl Drop for BackendHandle {
    fn drop(&mut self) {
        self.0.kill();
    }
}

/// One warm runtime instance. Owns the runtime API socket and the bootstrap.
#[derive(Debug)]
struct Instance {
    api: RuntimeApiHandle,
    backend: BackendHandle,
    last_used: Instant,
}

/// Pool of warm instances per `(function, qualifier)` key.
///
/// `acquire` fast-paths a warm instance and otherwise asks the backend to spawn
/// a new one. `release` returns the instance to the pool subject to
/// `max_warm`. `reap_idle` evicts instances older than `idle_timeout`.
#[derive(Debug)]
pub(crate) struct InstancePool {
    backend: Arc<dyn InstanceBackend>,
    max_warm: usize,
    idle_timeout: Duration,
    init_timeout: Duration,
    pools: Mutex<HashMap<PoolKey, Vec<Instance>>>,
}

impl InstancePool {
    pub(crate) fn new(
        backend: Arc<dyn InstanceBackend>,
        max_warm: usize,
        idle_timeout: Duration,
        init_timeout: Duration,
    ) -> Self {
        Self {
            backend,
            max_warm,
            idle_timeout,
            init_timeout,
            pools: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn key(req: &InvokeRequest) -> PoolKey {
        (req.function_name.clone(), req.qualifier.clone())
    }

    /// Run a single invocation against an acquired (or freshly spawned) instance.
    pub(crate) async fn invoke(&self, req: InvokeRequest) -> Result<InvokeResponse, ExecutorError> {
        let key = Self::key(&req);
        let mut instance = match self.try_acquire(&key) {
            Some(inst) => inst,
            None => self.spawn_new(&req).await?,
        };

        let request_id = uuid::Uuid::new_v4().to_string();
        let deadline = Instant::now() + req.timeout;
        let (resp_tx, resp_rx) = oneshot::channel();
        let job = Job {
            request_id: request_id.clone(),
            function_arn: req.function_arn.clone(),
            deadline,
            payload: req.payload.clone(),
            response_tx: resp_tx,
        };
        instance
            .api
            .submit(job)
            .await
            .map_err(|e| ExecutorError::Io(e.to_string()))?;

        let result = match tokio::time::timeout(req.timeout, resp_rx).await {
            Ok(Ok(r)) => r,
            Ok(Err(_)) => {
                // Bootstrap died before responding.
                instance.backend.kill();
                return Err(ExecutorError::RuntimeExited(
                    "bootstrap exited mid-invocation".to_owned(),
                ));
            }
            Err(_) => {
                instance.backend.kill();
                return Err(ExecutorError::Timeout(req.timeout));
            }
        };

        instance.last_used = Instant::now();
        self.release(key, instance);

        match result {
            RuntimeResult::Success(payload) => Ok(InvokeResponse {
                status: 200,
                payload,
                function_error: None,
                log_tail: None,
                executed_version: req.qualifier,
            }),
            RuntimeResult::Error(payload) => Ok(InvokeResponse {
                status: 200,
                payload,
                function_error: Some("Unhandled".to_owned()),
                log_tail: None,
                executed_version: req.qualifier,
            }),
            RuntimeResult::InitError(payload) => {
                let msg = String::from_utf8_lossy(&payload).into_owned();
                Err(ExecutorError::InitFailed(msg))
            }
        }
    }

    fn try_acquire(&self, key: &PoolKey) -> Option<Instance> {
        let mut pools = self.pools.lock();
        let bucket = pools.get_mut(key)?;
        bucket.pop()
    }

    fn release(&self, key: PoolKey, instance: Instance) {
        let mut pools = self.pools.lock();
        let bucket = pools.entry(key).or_default();
        if bucket.len() >= self.max_warm {
            // Pool full — drop (kill) on background task, don't block.
            tokio::spawn(async move {
                drop(instance);
            });
        } else {
            bucket.push(instance);
        }
    }

    async fn spawn_new(&self, req: &InvokeRequest) -> Result<Instance, ExecutorError> {
        let api = runtime_api::start()
            .await
            .map_err(|e| ExecutorError::Io(format!("bind runtime api: {e}")))?;
        let addr = api.addr();
        let mut init_err_rx = api.take_init_error_rx().await;

        // Race: backend spawn + first /next poll.  We don't observe /next here
        // directly — we rely on either submit landing on a polling bootstrap
        // OR an `/init/error` arriving.  To keep liveness, spawn the backend
        // within the init window and watch the init-error channel for a
        // fast-fail signal.
        let backend = self.backend.spawn(req, addr).await?;
        let inst = Instance {
            api,
            backend,
            last_used: Instant::now(),
        };
        // If the bootstrap failed init, surface that promptly rather than
        // waiting for the invocation timeout.
        if let Some(rx) = init_err_rx.take() {
            let init_timeout = self.init_timeout;
            tokio::spawn(async move {
                if let Ok(Ok(body)) = tokio::time::timeout(init_timeout, rx).await {
                    warn!(error = %String::from_utf8_lossy(&body), "lambda bootstrap reported init error");
                }
            });
        }
        debug!(function = %req.function_name, addr = %addr, "spawned new lambda instance");
        Ok(inst)
    }

    /// Reap instances idle for longer than `idle_timeout`. Returns count.
    pub(crate) fn reap_idle(&self) -> usize {
        let now = Instant::now();
        let idle = self.idle_timeout;
        let mut killed = 0usize;
        let mut pools = self.pools.lock();
        for bucket in pools.values_mut() {
            let mut keep = Vec::with_capacity(bucket.len());
            while let Some(inst) = bucket.pop() {
                if now.duration_since(inst.last_used) > idle {
                    killed += 1;
                    drop(inst);
                } else {
                    keep.push(inst);
                }
            }
            *bucket = keep;
        }
        killed
    }

    /// Drain and kill every instance in every pool.
    pub(crate) fn shutdown(&self) {
        let pools = std::mem::take(&mut *self.pools.lock());
        for (_, bucket) in pools {
            for inst in bucket {
                drop(inst);
            }
        }
    }
}

/// Spawn a periodic idle-reaper background task. Stops when `cancel` flips.
pub(crate) fn spawn_reaper(
    pool: Arc<InstancePool>,
    interval: Duration,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        tick.tick().await; // skip the immediate tick
        loop {
            tokio::select! {
                _ = cancel.changed() => break,
                _ = tick.tick() => {
                    let n = pool.reap_idle();
                    if n > 0 {
                        debug!(reaped = n, "lambda idle reaper");
                    }
                }
            }
        }
    })
}
