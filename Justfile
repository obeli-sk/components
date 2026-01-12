build:
    just run-all build

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
