# Justfile for mmbot project

# Auto-detect container runtime: podman > docker
compose := `command -v podman > /dev/null 2>&1 && echo "podman compose" || echo "docker compose"`

# Default recipe to display help information
default:
    @just --list

# Build the project
build:
    cargo build

# Run clippy with warnings as errors
clippy:
    cargo clippy -- -D warnings

# Format code with rustfmt
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Run all checks (fmt, clippy, build)
check: fmt-check clippy build
    @echo "✅ All checks passed!"

# Start test environment (Mattermost + Postgres)
test-env-start:
    @echo "🚀 Starting test environment..."
    {{compose}} up -d
    @echo "⏳ Waiting for Mattermost to be ready..."
    @for i in {1..60}; do \
        if curl -sf http://localhost:8065/api/v4/system/ping > /dev/null 2>&1; then \
            echo "✅ Mattermost is ready!"; \
            exit 0; \
        fi; \
        echo "   Attempt $$i/60..."; \
        sleep 2; \
    done; \
    echo "❌ Mattermost failed to start"; \
    exit 1

# Stop test environment
test-env-stop:
    @echo "🛑 Stopping test environment..."
    {{compose}} down -v
    @echo "✅ Test environment cleaned up"

# Show logs from test environment
test-env-logs:
    {{compose}} logs -f

# Run integration tests (requires test environment to be running)
test-integration:
    @echo "🧪 Running integration tests..."
    cargo test --test integration_tests -- --nocapture --test-threads=1

# Run all tests (unit + integration)
test-all:
    @echo "🧪 Running all tests..."
    cargo test --workspace

# Setup test environment and run integration tests
test: test-env-start test-integration

# Full test cycle: start env, run tests, stop env
test-full: test-env-start
    cargo test --test integration_tests -- --nocapture --test-threads=1 || (just test-env-stop && exit 1)
    just test-env-stop
