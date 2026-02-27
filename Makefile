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

update-submodule:
	@git submodule update --init --recursive --remote

.PHONY: build check test fmt clippy audit deny run-s3 release update-submodule
