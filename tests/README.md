# Worktree Agent Tests

This directory contains integration tests for worktree-agent.

## Integration Tests

### Merge Functionality Test

Location: `tests/integration/test-merge.sh`

Comprehensive test suite for merge functionality, covering:
- Default branch detection (worktrunk-style)
- Merge to detected default branch (main/master)
- `--target` flag for explicit merge targets
- Commands running from inside worktrees
- Worktree cleanup after merge
- Content verification
- Git history integrity

**Run the test:**

```bash
# Build the binary first
cargo build --release

# Run the test
./tests/integration/test-merge.sh
```

**What it tests:**

1. ✅ Default branch detection (main/master)
2. ✅ Merge without --target (merges to main)
3. ✅ Merge with --target (merges to specific branch)
4. ✅ Commands work from inside worktrees
5. ✅ Worktree cleanup after successful merge
6. ✅ Content is correctly merged
7. ✅ Git history is maintained

**Test environment:**

- Creates isolated test repository in `/tmp/test-wta-merge-suite-<PID>`
- Zero impact on the worktree-agent repository
- Automatically cleans up after completion

## Running All Tests

```bash
# Build
cargo build --release

# Run integration tests
./tests/integration/test-merge.sh

# Run Rust unit tests (when available)
cargo test
```

## Adding New Tests

When adding new integration tests:

1. Create executable bash script in `tests/integration/`
2. Use `/tmp` for test repositories (with PID for uniqueness)
3. Add cleanup trap to remove test directories on exit
4. Follow the existing test structure (setup, tests, verification, cleanup)
5. Update this README with test description

## CI/CD Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Build
  run: cargo build --release

- name: Run integration tests
  run: ./tests/integration/test-merge.sh
```

## Related Beads

- `worktree-agent-v9t`: Fix base_branch bug
- `worktree-agent-w5d`: Fix find_repo_root bug
- `worktree-agent-x8p`: Design decision for default branch merge
- `worktree-agent-ox2`: Add --target flag
- `worktree-agent-jdj`: Add integration tests
