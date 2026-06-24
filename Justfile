all-build:
	just run-all build

all-verify-local:
	just run-all verify-local

all-verify-oci:
	just run-all-with-oci-config verify-oci

all-verify: all-verify-local all-verify-oci

all-test *args:
	cargo nextest run --workspace {{args}}

all-test-e2e *args:
	./scripts/test-e2e.sh {{args}}

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

# private
run-all-with-oci-config *args:
	set -e && find . -name obelisk-oci.toml | while read -r jf; do \
		dir=$(dirname "$jf"); \
		echo "==> $dir ({{args}})"; \
		(cd "$dir" && just {{args}}); \
	done
