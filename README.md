# Obelisk Components

Reusable [WASIp2](https://github.com/WebAssembly/WASI/tree/main/wasip2) components for [Obelisk](https://obeli.sk/), a deterministic workflow engine for durable execution.

## Components

### Activities

Activities are components that perform side effects (HTTP calls, database operations, etc.).

| Component | Description |
|-----------|-------------|
| [activity-docker](docker/activity-docker) | Docker container management |
| [activity-fly-http](fly/activity-fly-http) | Fly.io API (apps, machines, secrets, volumes) |
| [activity-github-graphql](github/activity-github-graphql) | GitHub GraphQL API (account info, stargazers) |
| [activity-http-generic](http/activity-http-generic) | Generic HTTP client |
| [activity-obelisk-client-http](obelisk/activity-obelisk-client-http) | Obelisk API client |
| [activity-openai-responses](openai/activity-openai-responses) | OpenAI Responses API |
| [activity-postmark-email](postmark/activity-postmark-email) | Postmark email API |
| [activity-sendgrid-email](sendgrid/activity-sendgrid-email) | SendGrid email API |

### Webhooks

Webhooks are HTTP endpoint handlers that receive external events.

| Component | Description |
|-----------|-------------|
| [webhook-fly-secrets-updater](fly/webhook-fly-secrets-updater) | Fly.io secrets webhook handler |

## Development Setup

### Using Nix (Recommended)

This project uses [Nix flakes](https://nixos.wiki/wiki/Flakes) to provide a reproducible development environment.

**1. Install Nix with flakes enabled:**

```bash
# Using the Determinate Systems installer (recommended)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Or manually enable flakes in an existing Nix installation:

```bash
echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
sudo systemctl restart nix-daemon.service
```

**2. Configure Garnix cache for faster builds:**

```bash
cat << 'EOF' | sudo tee -a /etc/nix/nix.conf
extra-substituters = https://cache.garnix.io
extra-trusted-public-keys = cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=
EOF
sudo systemctl restart nix-daemon.service
```

**3. Enter the development shell:**

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

# Install obelisk CLI
cargo install obelisk --version <version-from-dev-deps.txt>
```

## Building

```bash
# Build all activities
just all-build

# Build a specific component
cargo build --target wasm32-wasip2 --release -p activity-openai-responses
```

## Testing

### Unit Tests

Unit tests run without external dependencies:

```bash
# Run all unit tests
cargo nextest run --workspace

# Or with just
just all-test
```

### End-to-End Tests

E2E tests verify components work correctly against real (or mock) APIs:

```bash
# Run all e2e tests with mock servers
./scripts/test-e2e.sh all

# Run specific component tests
./scripts/test-e2e.sh openai    # Uses mock server
./scripts/test-e2e.sh sendgrid  # Uses mock server
./scripts/test-e2e.sh postmark  # Uses mock server
./scripts/test-e2e.sh http      # Uses mock server
./scripts/test-e2e.sh github    # Uses real API (requires TEST_GITHUB_TOKEN)
./scripts/test-e2e.sh fly       # Uses real API (requires TEST_FLY_API_TOKEN)
```

**Environment variables for real API tests:**

- `TEST_GITHUB_TOKEN` - GitHub Personal Access Token
- `TEST_GITHUB_LOGIN` - GitHub username (default: obeli-sk)
- `TEST_FLY_API_TOKEN` - Fly.io API token
- `TEST_FLY_ORG` - Fly.io organization slug (default: personal)

## Using with Obelisk

Each component includes an `obelisk-local.toml` for local development:

```bash
cd openai/activity-openai-responses
cargo build --target wasm32-wasip2 --release
obelisk server run --config ./obelisk-local.toml
```

Submit executions from another terminal:

```bash
obelisk execution submit --follow \
  obelisk-components:openai-responses/api/create-simple \
  -- '{"model":"gpt-4o-mini","system_instructions":"You are helpful.","input":"Hello!"}'
```

## License

See [LICENSE](LICENSE) for details.
