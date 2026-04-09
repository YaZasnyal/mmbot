# Integration Tests for mattermost-bot

This directory contains integration tests that verify the bot works correctly with a real Mattermost instance.

## Prerequisites

- Docker and Docker Compose (or Podman)
- [just](https://github.com/casey/just) command runner (optional, but recommended)

## Running Tests

### Option 1: Using `just` (Recommended)

From the project root:

```bash
# Start test environment, run tests, and stop environment
just test-full

# Or run individual steps:
just test-env-start    # Start Mattermost
just test-integration  # Run tests (requires env to be running)
just test-env-stop     # Stop Mattermost
```

### Option 2: Manual Setup

1. **Start the test environment:**
   ```bash
   docker-compose up -d
   ```

2. **Wait for Mattermost to be ready:**
   ```bash
   # Wait until this returns successfully
   curl -sf http://localhost:8065/api/v4/system/ping
   ```

3. **Run the tests:**
   ```bash
   cargo test --test integration_tests -- --nocapture --test-threads=1
   ```

4. **Stop the environment:**
   ```bash
   docker-compose down -v
   ```

## Test Structure

- **`docker-compose.yml`** (in project root) - Defines Mattermost, PostgreSQL, and test-db services
- **`common/mod.rs`** - Helper functions for setting up test resources (users, bots, teams, channels)
- **`integration_tests.rs`** - Actual integration test cases
- **`smoke_test.rs`** - Simple smoke test to verify testcontainers works

## How Tests Work

1. Tests expect Mattermost to be running externally (via `docker-compose up -d` from project root)
2. Each test creates its own isolated resources:
   - Admin user (with unique name)
   - Team
   - Channel
   - Bot account
3. Tests verify bot behavior by:
   - Connecting the bot to Mattermost
   - Triggering events (posting messages, adding reactions)
   - Checking that the bot receives and processes these events correctly
4. Resources are isolated by using unique names (UUID-based)

## Environment Variables

- `MATTERMOST_URL` - Override Mattermost URL (default: `http://localhost:8065`)
- `MM_ADMIN_USER` - Existing admin username (optional, for reusing existing Mattermost instance)
- `MM_ADMIN_PASS` - Existing admin password (optional, for reusing existing Mattermost instance)

**Note**: If `MM_ADMIN_USER` and `MM_ADMIN_PASS` are not set, tests will create a new admin user automatically. This requires `MM_SERVICESETTINGS_ENABLEOPENSERVER=true` in Mattermost configuration.

## CI/CD Integration

### GitHub Actions

```yaml
- name: Start Mattermost
  run: docker-compose up -d

- name: Wait for Mattermost
  run: |
    for i in {1..60}; do
      if curl -sf http://localhost:8065/api/v4/system/ping; then
        echo "Mattermost is ready"
        exit 0
      fi
      sleep 2
    done
    exit 1

- name: Run tests
  run: cargo test --test integration_tests -- --nocapture --test-threads=1

- name: Stop Mattermost
  if: always()
  run: docker-compose down -v
```

### GitLab CI

```yaml
test:
  services:
    - docker:dind
  before_script:
    - docker-compose up -d
    - # wait for ready
  script:
    - cargo test --test integration_tests
  after_script:
    - docker-compose down -v
```

## Troubleshooting

### Mattermost fails to start

Check logs:
```bash
docker-compose logs
```

Common issues:
- Port 8065 already in use
- PostgreSQL not ready (should be handled by healthchecks)
- Insufficient resources

### Tests timeout

- Increase wait times in tests
- Check if Mattermost is actually responding: `curl http://localhost:8065/api/v4/system/ping`
- Verify bot can authenticate

### Resource conflicts

If tests fail with "username already exists" or similar:
```bash
# Clean up and restart environment
just test-env-stop
just test-env-start
```

## Writing New Tests

Example test structure:

```rust
#[tokio::test]
async fn test_my_feature() -> Result<()> {
    // Setup - creates all resources automatically (admin, team, channel, bot)
    let env = MattermostTestEnv::new().await?;
    
    // Create and run bot
    let bot = Bot::with_config((*env.bot_config()).clone())?
        .with_plugin(MyTestPlugin {});
    
    // Get admin HTTP client for API calls
    let admin = env.http_client(None);
    
    // Send a message
    admin.post_message(&env.channel_id, "Hello!").await?;
    
    // Add a reaction
    let post = admin.post_message(&env.channel_id, "React to this").await?;
    let post_id = post["id"].as_str().unwrap();
    admin.add_reaction(post_id, "smile").await?;
    
    // ... test logic ...
    
    Ok(())
}
```

That's it! Just 3 lines to get a fully configured test environment.

## Performance

- Starting Mattermost: ~10-30 seconds
- Running a single test: ~5-10 seconds
- Full test suite: depends on number of tests

To improve performance, consider:
- Running tests in parallel (requires separate Mattermost instances or careful resource isolation)
- Reusing the same Mattermost instance across test runs (requires cleanup between tests)
