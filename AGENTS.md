# AI Agent Guidelines for Obelisk Components

This document provides guidelines for AI agents working on this repository.

**Obelisk** is a deterministic workflow engine for durable execution. Learn more at [obeli.sk](https://obeli.sk/).

## Development Environment

### Using Nix (Recommended)

This project uses Nix flakes to manage all dependencies including Rust, cargo, and other tools.

**Installing Nix with flakes enabled:**

```bash
# Using the Determinate Systems installer (recommended)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# Or manually enable flakes in an existing Nix installation:
echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
sudo systemctl restart nix-daemon.service
```

**Configure Garnix cache for faster builds:**

```bash
cat << 'EOF' | sudo tee -a /etc/nix/nix.conf
extra-substituters = https://cache.garnix.io
extra-trusted-public-keys = cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g
EOF
sudo systemctl restart nix-daemon.service
```

**Enter the development shell:**

```bash
nix develop
```

This provides all necessary tools: Rust toolchain, wasmtime, wasm-tools, etc.

### Without Nix

If Nix is unavailable, install tools matching versions in `dev-deps.txt`:

```bash
# Check required versions
cat dev-deps.txt

# Install Rust toolchain per rust-toolchain.toml
rustup default 1.92
rustup target add wasm32-wasip2

# Install wasmtime (check dev-deps.txt for version)
curl https://wasmtime.dev/install.sh -sSf | bash

# Install obelisk CLI (prefer binstall for faster installation)
cargo binstall obelisk --version <version-from-dev-deps.txt>
# Or if binstall is not available:
cargo install obelisk --version <version-from-dev-deps.txt>
```

## Project Structure

### Naming Conventions

- **Activities** (components responsible for side effects): Must start with `activity-`
  - Example: `fly/activity-fly-http`, `openai/activity-openai-responses`
- **Webhooks** (HTTP endpoint handlers): Must start with `webhook-`
  - Example: `fly/webhook-fly-secrets-updater`

### Activity Components

Activities are WASIp2 components that perform side effects. Each activity lives in its own directory:

```
<category>/activity-<name>/
├── .cargo/config.toml    # Build target and test runner config
├── .envrc-example        # Example env vars (if activity requires them)
├── Cargo.toml            # Uses workspace dependencies
├── README.md
├── obelisk-local.toml    # Local Obelisk config (path to wasm)
├── src/
│   └── lib.rs
└── wit/
    ├── impl.wit                              # World definition (any:any)
    ├── <package-name>/                       # Main exported interface
    │   └── <interface>.wit
    └── deps/
        └── <package-name> -> ../<package-name>  # Symlink to main package
```

If the activity requires environment variables (API keys, tokens, etc.), include a `.envrc-example` file showing the required variables and document them in the README.

### WIT File Organization

1. **Main package directory** (`wit/<package-name>/`): Contains the interface definition. This is the canonical source, readable by humans.

2. **deps symlink** (`wit/deps/<package-name>`): A symlink to the main package. Required for wit-bindgen to resolve dependencies.

3. **impl.wit**: Defines the world that exports your interface:
   ```wit
   package any:any;

   world any {
       export obelisk-components:<your-package>/<interface>@0.1.0;
   }
   ```

### Cargo.toml

Use workspace dependencies:

```toml
[package]
name = "activity-<name>"
description = "Activity that does X."

authors.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
version.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
serde.workspace = true
serde_json.workspace = true
wit-bindgen.workspace = true
wstd.workspace = true

[build-dependencies]
anyhow.workspace = true
wit-bindgen-rust.workspace = true
```

### WIT Bindings Generation

**Prefer using `build.rs`** over the `wit_bindgen::generate!` macro for generating WIT bindings. This provides better IDE support and more control over code generation.

**build.rs:**
```rust
use anyhow::Result;
use wit_bindgen_rust::Opts;

fn main() -> Result<()> {
    Opts {
        generate_all: true,
        ..Default::default()
    }
    .build()
    .generate_to_out_dir(None)?;

    Ok(())
}
```

**src/lib.rs:**
```rust
mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

use generated::export;
use generated::exports::obelisk_components::your_package::api::*;

struct Component;
export!(Component with_types_in generated);

impl Guest for Component {
    // ...
}
```

### .cargo/config.toml

Configure wasm32-wasip2 as default target and wasmtime as test runner:

```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "wasmtime run -Scli -Shttp --env YOUR_API_KEY --env TEST_YOUR_API_KEY"
```

## Error Handling for Obelisk

**Critical**: All activity errors MUST use a `variant` type with an `execution-failed` case:

```wit
variant api-error {
    /// Required for Obelisk - indicates timeout or trap in last retry
    execution-failed,
    /// Your other error cases
    configuration-error(string),
    request-failed(string),
    api-error(error-details),
}
```

The `execution-failed` variant is used by Obelisk to handle:
- Execution timeouts
- Component traps/panics
- Retry exhaustion

**Do not use `result<T, string>`** for activity functions - use structured error variants.

## Testing

### Test Structure

Tests run in wasmtime via the `.cargo/config.toml` runner:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const ENV_API_KEY: &str = "API_KEY";

    fn set_up() {
        // Copy TEST_* env var to the actual env var
        let test_token = std::env::var(format!("TEST_{ENV_API_KEY}"))
            .expect("TEST_API_KEY must be set");
        unsafe { std::env::set_var(ENV_API_KEY, test_token) };
    }

    #[test]
    fn unit_test_no_external_deps() {
        // Runs without setup
    }

    #[test]
    #[ignore]  // Integration tests are ignored by default
    fn integration_test_requires_api_key() {
        set_up();
        // Test with real API
    }
}
```

### Running Tests

```bash
# Unit tests only (from activity directory)
cargo test

# Integration tests (requires API keys)
export TEST_YOUR_API_KEY="..."
cargo test -- --ignored --nocapture

# All tests
cargo test -- --include-ignored
```

## HTTP Requests with wstd

Use `wstd` for async HTTP in WASIp2 components:

```rust
use wstd::http::{Body, Client, Request};
use wstd::runtime::block_on;

async fn make_request() -> Result<String, Error> {
    let req = Request::post("https://api.example.com/endpoint")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json_body))?;
    
    let resp = Client::new().send(req).await?;
    let body = resp.into_body().contents().await?;
    Ok(String::from_utf8_lossy(&body).to_string())
}

// In your Guest impl:
fn my_function() -> Result<String, ApiError> {
    block_on(make_request())
}
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --target wasm32-wasip2` and fix all warnings
- Follow existing patterns in the repository

## Obelisk Configuration

### obelisk-local.toml

Each activity should have an `obelisk-local.toml` for local development:

```toml
api.listening_addr = "127.0.0.1:5005"
webui.listening_addr = "127.0.0.1:8080"

[[activity_wasm]]
name = "activity_your_name"
location.path = "${OBELISK_TOML_DIR}/../../target/wasm32-wasip2/release/activity_your_name.wasm"
exec.lock_expiry.seconds = 5
env_vars = ["YOUR_API_KEY"]
forward_stdout = "stderr"
forward_stderr = "stderr"
```

Run locally with:
```bash
cargo build --release
obelisk server run --config ./obelisk-local.toml
```

### obelisk-oci.toml

The `obelisk-oci.toml` file is **auto-generated** by a GitHub Action that is triggered manually. Do not create or edit this file manually.

## Adding a New Activity

1. Create directory: `<category>/activity-<name>/`
2. Add to workspace `Cargo.toml` members
3. Create `.cargo/config.toml` with target and runner
4. Create `Cargo.toml` using workspace dependencies (include build-dependencies)
5. Create `build.rs` for WIT binding generation
6. Create WIT interface in `wit/<package-name>/<interface>.wit`
7. Create `wit/impl.wit` exporting your interface
8. Create symlink: `wit/deps/<package-name> -> ../<package-name>`
9. Implement in `src/lib.rs` (use generated module pattern)
10. Add tests
11. Add `README.md` (document required env vars if any)
12. Add `obelisk-local.toml` for local development
13. Add `.envrc-example` if the activity requires environment variables
