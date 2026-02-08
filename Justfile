all-build: build-activities build-webhooks

# private
build-activities:
	set -xe && cargo build --target=wasm32-wasip2 --profile=release_activity \
		$(cargo metadata --no-deps --format-version=1 \
		| jq -r '.packages[].name | select(startswith("activity-")) | "-p \(. )"' \
		| xargs)

# private
build-webhooks:
	set -xe && cargo build --target=wasm32-wasip2 --profile=release_webhook \
		$(cargo metadata --no-deps --format-version=1 \
		| jq -r '.packages[].name | select(startswith("webhook-")) | "-p \(. )"' \
		| xargs)

all-verify-local:
	just run-all verify-local

all-verify-oci:
	just run-all verify-oci

all-verify: all-verify-local all-verify-oci

all-test *args:
	cargo nextest run --workspace {{args}}

all-push target:
	just run-all push {{target}}

all-push-dryrun:
	just all-push dryrun

# private
run-all *args:
	set -e && find . -name obelisk-local.toml | while read -r jf; do \
		dir=$(dirname "$jf"); \
		echo "==> $dir ({{args}})"; \
		(cd "$dir" && just {{args}}); \
	done
