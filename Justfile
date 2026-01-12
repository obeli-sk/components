build: build-activities build-webhooks

build-activities:
	set -xe && cargo build --target=wasm32-wasip2 --profile=release_activity \
		$(cargo metadata --no-deps --format-version=1 \
		| jq -r '.packages[].name | select(startswith("activity-")) | "-p \(. )"' \
		| xargs)

build-webhooks:
	set -xe && cargo build --target=wasm32-wasip2 --profile=release_webhook \
		$(cargo metadata --no-deps --format-version=1 \
		| jq -r '.packages[].name | select(startswith("webhook-")) | "-p \(. )"' \
		| xargs)

verify-local:
	just run-all verify-local

verify-oci:
	just run-all verify-oci

verify: verify-local verify-oci

test *args:
	cargo nextest run --workspace {{args}}

run-all target:
	set -e && find . -name Justfile -not -path "./Justfile" | while read -r jf; do \
		dir=$(dirname "$jf"); \
		echo "==> $dir ({{target}})"; \
		(cd "$dir" && just "{{target}}"); \
	done
