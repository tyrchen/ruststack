build:
	@cargo build

check:
	@cargo check --all-targets --all-features

test:
	@cargo nextest run --all-features

fmt:
	@cargo +nightly fmt

clippy:
	@cargo clippy --all-targets --all-features -- -D warnings

audit:
	@cargo audit

deny:
	@cargo deny check

run-s3:
	@cargo run -p ruststack-s3-server

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -n -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

codegen:
	@cd codegen && cargo run
	@cargo +nightly fmt -p ruststack-s3-model

integration:
	@cargo test -p ruststack-s3-integration -- --ignored

mint: mint-start mint-run

mint-build:
	@cargo build --release -p ruststack-s3-server

mint-start: mint-build
	@echo "Starting RustStack S3 server..."
	@S3_SKIP_SIGNATURE_VALIDATION=true \
		GATEWAY_LISTEN=0.0.0.0:4566 \
		LOG_LEVEL=warn \
		cargo run --release -p ruststack-s3-server &
	@for i in $$(seq 1 30); do \
		if curl -sf http://127.0.0.1:4566/_localstack/health > /dev/null 2>&1; then \
			echo "Server is ready"; \
			break; \
		fi; \
		if [ "$$i" -eq 30 ]; then \
			echo "Server did not start within 30s"; \
			exit 1; \
		fi; \
		sleep 1; \
	done

CONTAINER_CMD := $(shell command -v docker 2>/dev/null || command -v podman 2>/dev/null)
# macOS containers can't use --network host; use host.containers.internal instead.
MINT_SERVER_ENDPOINT := $(shell if [ "$$(uname)" = "Darwin" ]; then echo "host.containers.internal:4566"; else echo "127.0.0.1:4566"; fi)
MINT_NETWORK := $(shell if [ "$$(uname)" = "Darwin" ]; then echo ""; else echo "--network host"; fi)

mint-run:
	@mkdir -p /tmp/mint-logs
	$(CONTAINER_CMD) run --rm $(MINT_NETWORK) \
		-e SERVER_ENDPOINT=$(MINT_SERVER_ENDPOINT) \
		-e ACCESS_KEY=test \
		-e SECRET_KEY=test \
		-e ENABLE_HTTPS=0 \
		minio/mint:latest 2>&1 | tee /tmp/mint-logs/mint-output.txt || true
	@echo ""
	@PASS_COUNT=$$(grep -oE 'Executed [0-9]+' /tmp/mint-logs/mint-output.txt | grep -oE '[0-9]+' || echo "0"); \
		FAIL_COUNT=$$(grep -c '"status": "FAIL"' /tmp/mint-logs/mint-output.txt || true); \
		echo "Mint results: $$PASS_COUNT passed, $$FAIL_COUNT failed"

mint-stop:
	@pkill -f "ruststack-s3-server" 2>/dev/null || true
	@echo "Server stopped"

update-submodule:
	@git submodule update --init --recursive --remote

.PHONY: build check test fmt clippy audit deny run-s3 release update-submodule codegen integration mint mint-build mint-start mint-run mint-stop
