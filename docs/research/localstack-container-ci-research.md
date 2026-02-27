# LocalStack Container & CI Research

Research into how LocalStack operates as a Docker container, how it serves all AWS services through a single port (4566), how the official GitHub Action works, and how CI environments consume it.

## 1. How the Official LocalStack Container Works

### 1.1 Docker Image Architecture

LocalStack publishes two Docker images:

- **`localstack/localstack`** -- The full community image with all AWS services.
- **`localstack/localstack:s3`** (built from `Dockerfile.s3`) -- A stripped-down image containing only S3.

Both are built on `python:3.13.12-slim-trixie` and follow a multi-stage build:

1. **base** -- Installs runtime OS packages (curl, openssl, Node.js 22, etc.), creates the `localstack` user, sets up filesystem hierarchy under `/opt/code/localstack/`, and installs the entrypoint script.
2. **builder** -- Installs build dependencies (gcc, g++), creates a Python venv, and installs runtime pip dependencies.
3. **final** -- Copies the venv from builder, installs LocalStack core as an editable package, generates the service catalog cache, and pre-installs packages like `lambda-runtime`, `jpype-jsonata`, and `dynamodb-local`.

Key Dockerfile directives from the full image (`Dockerfile`):

```dockerfile
# Port exposure
EXPOSE 4566 4510-4559 5678

# Health check
HEALTHCHECK --interval=10s --start-period=15s --retries=5 --timeout=10s \
  CMD /opt/code/localstack/.venv/bin/localstack status services --format=json

# Volume
VOLUME /var/lib/localstack

# Entrypoint
ENTRYPOINT ["docker-entrypoint.sh"]
```

The S3-only image (`Dockerfile.s3`) additionally sets these environment variables:

```dockerfile
ENV EAGER_SERVICE_LOADING=1
ENV SERVICES=s3
ENV GATEWAY_SERVER=twisted
ENV DNS_ADDRESS=false
```

### 1.2 Entrypoint Script (`bin/docker-entrypoint.sh`)

The entrypoint script performs these steps in order:

1. **Warning messages** -- Displays warnings if running community edition or if `LOCALSTACK_AUTH_TOKEN` is set in the community image.
2. **Environment variable stripping** -- Strips the `LOCALSTACK_` prefix from environment variables (e.g., `LOCALSTACK_DEBUG=1` becomes `DEBUG=1`), except for `LOCALSTACK_HOST` and `LOCALSTACK_HOSTNAME`.
3. **Log directory creation** -- Creates `/var/lib/localstack/logs` if it does not exist.
4. **Virtual environment activation** -- Sources `/opt/code/localstack/.venv/bin/activate`.
5. **Boot init hooks** -- If `/etc/localstack/init/boot.d` exists, runs `python3 -m localstack.runtime.init BOOT` to execute boot-stage init scripts.
6. **Supervisor launch** -- Calls `exec localstack-supervisor` (uses `exec` so signals propagate correctly).

### 1.3 Supervisor (`bin/localstack-supervisor`)

The supervisor is a Python script that acts as a mini init system:

- Starts the actual LocalStack process via `python -m localstack.runtime.main` (configurable via `LOCALSTACK_SUPERVISOR_COMMAND`).
- **Signal handling**:
  - `SIGTERM` / `SIGINT` -- Terminates the LocalStack process and then exits.
  - `SIGUSR1` -- Terminates the current LocalStack process and starts a new one (restart).
  - `SIGALRM` -- Used internally for timeout on graceful shutdown.
- If the child process exits on its own (not from SIGUSR1), the supervisor exits with the same exit code.
- Configurable shutdown timeout via `SHUTDOWN_TIMEOUT` env var (default 5 seconds). If the child does not stop within the timeout, it is `SIGKILL`ed.

### 1.4 Runtime Startup (`localstack.runtime.main`)

The runtime main module:

1. Calls `setup_logging_from_config()`.
2. Initializes the runtime via `current.initialize_runtime()`, which:
   - Fires `on_runtime_create` hooks.
   - Loads `Components` plugins from the `localstack.runtime.components` namespace.
   - Creates a `LocalstackRuntime` instance.
3. Sets up signal handlers for `SIGINT` and `SIGTERM`.
4. Calls `runtime.run()`, which:
   - Initializes the filesystem (creates config directories, clears temp).
   - Fires `on_runtime_start` hooks (which trigger lazy service loading).
   - Initializes the gateway server: creates an SSL cert, registers the `Gateway` with the `RuntimeServer`, and binds to addresses from `GATEWAY_LISTEN`.
   - Starts a daemon thread to monitor when the gateway port opens, then fires `on_runtime_ready` hooks and prints `"Ready."` (the ready marker).
   - Blocks on `runtime_server.run()` (the main HTTP server loop).

### 1.5 Port Binding

Port binding is configured via the `GATEWAY_LISTEN` environment variable:

- Default: `0.0.0.0:4566` (from `constants.DEFAULT_PORT_EDGE = 4566`).
- Supports multiple listen addresses: `GATEWAY_LISTEN=0.0.0.0:4566,0.0.0.0:443`.
- The `GATEWAY_SERVER` config (default: `"twisted"`) determines which HTTP server implementation is used (options: `twisted`, `hypercorn`, `werkzeug` for dev).

From the docker-compose.yml:

```yaml
ports:
  - "127.0.0.1:4566:4566"            # LocalStack Gateway
  - "127.0.0.1:4510-4559:4510-4559"  # external services port range
```

Port 4566 is the single gateway for ALL AWS API requests. Ports 4510-4559 are for "external service ports" used by services that create user-facing backends (e.g., RDS starts a PostgreSQL server, ElastiCache starts Redis).

## 2. How Services Are Routed Through the Gateway on Port 4566

### 2.1 The Gateway Architecture

All HTTP requests to port 4566 flow through the `LocalstackAwsGateway`, which implements a chain-of-responsibility pattern:

```
HTTP Request --> RuntimeServer (Twisted/Hypercorn) --> Gateway.process(request, response)
                                                         |
                                                         v
                                                    HandlerChain
```

The `LocalstackAwsGateway` registers this request handler chain (in order):

1. `add_internal_request_params` -- Adds internal metadata to the request context.
2. `handle_runtime_shutdown` -- Rejects requests if the runtime is shutting down.
3. `metric_collector.create_metric_handler_item` -- Creates a metric tracking item.
4. `load_service_for_data_plane` -- Early data-plane service detection (e.g., Lambda Function URLs, S3 website endpoints).
5. `preprocess_request` -- Preprocesses the raw HTTP request.
6. `enforce_cors` -- Handles CORS preflight.
7. `content_decoder` -- Decodes content encoding.
8. `validate_request_schema` -- Validates request schema for public endpoints.
9. **`serve_localstack_resources`** -- Serves internal endpoints under `/_localstack/*` (including the health endpoint).
10. `serve_edge_router_rules` -- Custom edge routing rules (e.g., OpenSearch cluster endpoints).
11. **`parse_service_name`** -- Determines which AWS service the request is for.
12. `parse_pre_signed_url_request` -- Handles S3 pre-signed URLs.
13. `inject_auth_header_if_missing` -- Injects auth headers.
14. `add_region_from_header` -- Extracts region from the request.
15. `rewrite_region` -- Rewrites region if needed.
16. `add_account_id` -- Extracts AWS account ID.
17. `parse_trace_context` -- Parses X-Ray/tracing context.
18. **`parse_service_request`** -- Parses the AWS operation and parameters from the request.
19. `metric_collector.record_parsed_request` -- Records parsed request metrics.
20. `serve_custom_service_request_handlers` -- Custom service-specific handlers.
21. **`load_service`** -- Lazy-loads the service provider plugin if not yet loaded.
22. **`service_request_router`** -- Routes the parsed request to the correct service provider.
23. `EmptyResponseHandler(404)` -- Fallback if nothing handled the request.

### 2.2 Service Name Detection (`ServiceNameParser`)

The `determine_aws_service_model()` function in `service_router.py` uses a multi-step heuristic to figure out which AWS service a request targets. It checks, in order:

1. **Signing name from Authorization header** -- The AWS SigV4 `Credential` field contains a signing name (e.g., `s3`, `dynamodb`, `sqs`). For ~75% of services, this uniquely identifies the service.
2. **Custom signing name + path prefix rules** -- For ambiguous signing names (e.g., `es` maps to both Elasticsearch and OpenSearch), path prefixes disambiguate (e.g., `/2015-01-01` = Elasticsearch, `/2021-01-01` = OpenSearch).
3. **X-Amz-Target header** -- For JSON/RPC-style protocols, the `X-Amz-Target` header contains `<TargetPrefix>.<Operation>`.
4. **Custom path-based rules** -- Special paths like `/2015-03-31/functions` map to Lambda; SQS queue URLs are detected by pattern.
5. **Host-based rules** -- `.lambda-url.` in the host maps to Lambda; `.s3-website.` maps to S3.
6. **Query/form-data `Action` parameter** -- For query protocol services (SQS, STS, IAM), the `Action=<OperationName>` parameter identifies the service.
7. **Conflict resolution** -- For ambiguous candidates (e.g., `timestream-query` vs `timestream-write`), manual rules pick a winner.
8. **Legacy S3 fallback** -- If nothing else matches, a set of heuristic rules tries S3 (e.g., `GET /<bucket>/<key>`, pre-signed URL parameters, `AWS id:key` auth headers).

### 2.3 Service Request Routing (`ServiceRequestRouter`)

Once the service is identified:

1. The service provider plugin is lazy-loaded if not already loaded.
2. A `Skeleton` wraps the provider's Python class and maps AWS operation names to handler functions.
3. The `ServiceRequestRouter` dispatches `ServiceOperation(service_name, operation_name)` to the corresponding `SkeletonHandler`.
4. The skeleton:
   - Parses the HTTP request into typed Python objects using botocore's service model.
   - Calls the provider's handler method.
   - Serializes the response back into the correct AWS protocol format (JSON, query, rest-xml, etc.).

### 2.4 Protocol Detection

LocalStack supports these AWS protocols:

- `smithy-rpc-v2-cbor` -- Detected via `Smithy-Protocol: rpc-v2-cbor` header.
- `json` -- Detected via `Content-Type: application/x-amz-json-*`.
- `query` / `ec2` -- Detected via `Content-Type: application/x-www-form-urlencoded` or `Action` query parameter.
- `rest-json` / `rest-xml` -- Catch-all for REST-style APIs.

## 3. Health Endpoint

### 3.1 The Docker HEALTHCHECK

The Dockerfile declares:

```dockerfile
HEALTHCHECK --interval=10s --start-period=15s --retries=5 --timeout=10s \
  CMD /opt/code/localstack/.venv/bin/localstack status services --format=json
```

This runs the LocalStack CLI's `status services` command every 10 seconds. Docker marks the container as "healthy" when this command exits 0. It allows a 15-second start period before starting checks.

### 3.2 The `/_localstack/health` HTTP Endpoint

The `HealthResource` class (in `localstack/services/internal.py`) handles `/_localstack/health`:

**GET `/_localstack/health`**

Returns a JSON object with the following structure:

```json
{
  "services": {
    "s3": "running",
    "sqs": "available",
    "dynamodb": "running",
    "lambda": "disabled",
    ...
  },
  "edition": "community",
  "version": "4.x.x"
}
```

Service states are defined in `ServiceState` enum:

| State | Value | Meaning |
|---|---|---|
| `UNKNOWN` | `"unknown"` | State has not been determined yet |
| `AVAILABLE` | `"available"` | Service is available but not yet started |
| `DISABLED` | `"disabled"` | Service is explicitly disabled |
| `STARTING` | `"starting"` | Service is currently starting up |
| `RUNNING` | `"running"` | Service is running and healthy |
| `STOPPING` | `"stopping"` | Service is shutting down |
| `STOPPED` | `"stopped"` | Service has been stopped |
| `ERROR` | `"error"` | Service failed to start or encountered an error |

If `?reload` is in the path, all services are actively health-checked before returning.

The `state` dict can be extended by PUT requests (used internally, e.g., for init script status tracking).

**POST `/_localstack/health`**

Accepts JSON body with an `action` field:

- `{"action": "restart"}` -- Signals the supervisor to restart LocalStack (sends SIGUSR1).
- `{"action": "kill"}` -- Triggers a graceful shutdown with exit code 0.

**HEAD `/_localstack/health`**

Returns `200 OK` with "ok" body. Useful for simple liveness probes.

### 3.3 Other Internal Endpoints

- `/_localstack/info` -- Returns session info, version, uptime, machine ID, system details.
- `/_localstack/plugins` -- Lists all loaded plugins.
- `/_localstack/init` -- Shows init script execution status.
- `/_localstack/init/<stage>` -- Shows init scripts for a specific stage (BOOT, START, READY, SHUTDOWN).
- `/_localstack/config` -- Read/update config at runtime (requires `ENABLE_CONFIG_UPDATES=1`).
- `/_localstack/diagnose` -- Full diagnostic dump (requires `DEBUG=1`).

### 3.4 Ready Marker

When the runtime is fully started and the gateway port is accepting connections, it prints `"Ready."` to stdout. This is the `READY_MARKER_OUTPUT` constant. The `localstack wait` CLI command streams the container's logs and blocks until it sees this line (or times out).

An alternative readiness signal is the file `/var/lib/localstack/localstack_ready` which can be used in Docker health checks:

```yaml
healthcheck:
  test: ["CMD", "test", "-f", "/var/lib/localstack/localstack_ready"]
```

## 4. How the Official GitHub Action (`localstack/setup-localstack`) Works

### 4.1 Action Overview

The `localstack/setup-localstack` GitHub Action is a composite action with modular sub-actions:

```
setup-localstack/
  action.yml          # Main action definition
  startup/
    action.yml        # Sub-action: starts LocalStack
  prepare/            # Setup preparation
  cloud-pods/         # Cloud Pods state management
  ephemeral/          # Ephemeral Instance handling
  local/              # Local state management
  finish/             # Cleanup operations
```

### 4.2 Key Inputs

| Input | Default | Description |
|---|---|---|
| `image-tag` | `"latest"` | Docker image tag to pull |
| `install-awslocal` | `"true"` | Install the `awslocal` CLI wrapper |
| `use-pro` | `"false"` | Use `localstack/localstack-pro` image |
| `configuration` | `""` | Config vars (e.g., `DEBUG=1,SERVICES=s3`) |
| `skip-startup` | `"false"` | Only install CLIs, do not start container |
| `skip-wait` | `"false"` | Skip waiting for LocalStack to be ready |
| `state-action` | `""` | `load`, `save`, `start`, `stop`, or empty |
| `state-backend` | `""` | `cloud-pods`, `ephemeral`, or `local` |
| `state-name` | `""` | State artifact name |
| `auto-load-pod` | `""` | Cloud Pod to load on startup |
| `extension-auto-install` | `""` | Extensions to auto-install |

### 4.3 Startup Sequence

The startup sub-action performs these steps:

1. **Select image**: If `use-pro` is true, uses `localstack/localstack-pro:${IMAGE_TAG}`; otherwise `localstack/localstack:${IMAGE_TAG}`.
2. **Pull image**: Runs `docker pull ${IMAGE_NAME} &` in the background.
3. **Start container**: Runs `eval "${CONFIGURATION} localstack start -d"`. The `CONFIGURATION` string is prepended as environment variable assignments (e.g., `DEBUG=1 SERVICES=s3 localstack start -d`).
4. **Wait for readiness**: Runs `localstack wait -t ${LS_WAIT_TIMEOUT:-30}`. This blocks until the container prints the `"Ready."` marker to stdout, with a default 30-second timeout. Can be skipped with `skip-wait: 'true'`.

The `localstack start -d` CLI command:
- Creates a Docker container from the image.
- Mounts the Docker socket (`/var/run/docker.sock`).
- Maps port 4566 and the external port range.
- Sets environment variables from the configuration.
- Starts the container in detached mode.

The `localstack wait` CLI command:
- Streams the container's logs.
- Blocks until it sees `"Ready."` in the output or times out.

### 4.4 Alternative: Docker Service Container Pattern

Instead of the official action, projects can use GitHub Actions' built-in service container syntax:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      localstack:
        image: localstack/localstack
        ports:
          - "4566:4566"
        options: >-
          --health-cmd "curl -f http://localhost:4566/_localstack/health"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
          --health-start-period 15s
        env:
          SERVICES: s3,sqs,dynamodb
```

GitHub Actions will wait for the health check to pass before running job steps. The `curl` command against `/_localstack/health` is a common choice; alternatively `awslocal s3 ls` can be used.

### 4.5 State Management

The action supports three state persistence mechanisms:

1. **Cloud Pods** -- Save/load infrastructure state snapshots to LocalStack Cloud. Use `state-backend: cloud-pods` with `state-action: load` or `save`.
2. **Ephemeral Instances** -- Temporary cloud-hosted LocalStack instances for PR previews. Use `state-backend: ephemeral` with `state-action: start`/`stop`.
3. **Local Artifacts** -- Export/import state as GitHub Actions artifacts between workflow runs.

## 5. Summary of Key Integration Points

For building a Rust-based LocalStack-compatible service, the critical integration points are:

| Component | Interface | What to Implement |
|---|---|---|
| **Port 4566** | Single HTTP endpoint | An HTTP server that listens on 4566 and routes all AWS service requests |
| **Service routing** | AWS SigV4 signing name, `X-Amz-Target`, path, host, `Action` param | Request parsing to determine target service and operation |
| **Health endpoint** | `GET /_localstack/health` | Return JSON with `{ "services": {...}, "edition": "...", "version": "..." }` |
| **Health HEAD** | `HEAD /_localstack/health` | Return `200 OK` for liveness probes |
| **Ready marker** | `"Ready."` on stdout | Print after gateway is accepting connections |
| **Ready file** | `/var/lib/localstack/localstack_ready` | Touch file when ready (alternative readiness signal) |
| **Docker HEALTHCHECK** | Exit code 0 when healthy | Can use `curl /_localstack/health` or custom CLI |
| **GATEWAY_LISTEN** | `<host>:<port>` env var | Configure bind address (default `0.0.0.0:4566`) |
| **SERVICES** | Comma-separated list | Filter which services to load |
| **Entrypoint** | Docker `ENTRYPOINT` | Container startup script with signal handling |

## Sources

- [localstack/setup-localstack GitHub Repository](https://github.com/localstack/setup-localstack)
- [Setup LocalStack -- GitHub Marketplace](https://github.com/marketplace/actions/setup-localstack)
- [LocalStack GitHub Actions Documentation](https://docs.localstack.cloud/aws/integrations/continuous-integration/github-actions/)
- [LocalStack Internal Endpoints Documentation](https://docs.localstack.cloud/aws/capabilities/networking/internal-endpoints/)
- [LocalStack External Service Port Range](https://docs.localstack.cloud/aws/capabilities/networking/external-port-range/)
- [LocalStack Docker Hub](https://hub.docker.com/r/localstack/localstack)
- Vendor code in `vendors/localstack/` (Dockerfile, docker-entrypoint.sh, runtime modules)
