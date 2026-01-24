#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Find the repository root (where this script's parent/parent is)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Try to find wta binary (release first, then debug)
if [ -x "$REPO_ROOT/target/release/wta" ]; then
    WTA="$REPO_ROOT/target/release/wta"
elif [ -x "$REPO_ROOT/target/debug/wta" ]; then
    WTA="$REPO_ROOT/target/debug/wta"
else
    echo -e "${RED}Error: wta binary not found. Run 'cargo build' or 'cargo build --release' first.${NC}"
    exit 1
fi

TEST_DIR="/tmp/test-wta-merge-suite-$$"  # Use PID for uniqueness

echo -e "${BLUE}=== WTA Merge Functionality Test Suite ===${NC}\n"
echo -e "Repository: $REPO_ROOT"
echo -e "Binary: $WTA"
echo -e "Test directory: $TEST_DIR\n"

# Clean up any previous test
rm -rf "$TEST_DIR"

# Trap to cleanup on exit
cleanup() {
    if [ -d "$TEST_DIR" ]; then
        echo -e "\n${YELLOW}Cleaning up test directory...${NC}"
        cd /tmp
        rm -rf "$TEST_DIR" "$TEST_DIR-wta-"*
    fi
}
trap cleanup EXIT

# Function to print test results
pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
}

fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    exit 1
}

info() {
    echo -e "${YELLOW}→${NC} $1"
}

# Test 1: Setup test repository
echo -e "\n${BLUE}Test 1: Setup test repository${NC}"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"
git init
git config user.email "test@example.com"
git config user.name "Test User"
echo "# Test Project" > README.md
git add README.md
git commit -m "Initial commit"
git branch -M main
pass "Created test repository with main branch"

# Test 2: Default branch detection
echo -e "\n${BLUE}Test 2: Default branch detection${NC}"
info "Testing default branch detection from local inference..."

# Check if main branch exists
if git rev-parse --verify refs/heads/main >/dev/null 2>&1; then
    pass "main branch exists and should be detected"
else
    fail "main branch should exist"
fi

# Test 3: Create a feature branch with changes
echo -e "\n${BLUE}Test 3: Create feature branch with changes${NC}"
git checkout -b feature-1
echo "Feature 1 code" > feature1.txt
git add feature1.txt
git commit -m "Add feature 1"
pass "Created feature-1 branch with changes"

# Test 4: Create worktree manually and set up agent state
echo -e "\n${BLUE}Test 4: Setup agent state for merge test${NC}"
git checkout main
git worktree add "$TEST_DIR-wta-1" feature-1

# Create .worktree-agents directory and state
mkdir -p "$TEST_DIR/.worktree-agents"
cat > "$TEST_DIR/.worktree-agents/state.json" << EOF
{
  "next_id": 2,
  "agents": [
    {
      "id": "1",
      "task": "Add feature 1",
      "branch": "feature-1",
      "base_branch": "main",
      "worktree_path": "$TEST_DIR-wta-1",
      "tmux_session": "wta-test",
      "tmux_window": "1",
      "status": "completed",
      "provider": "claude",
      "launched_at": "2026-01-24T10:00:00Z",
      "completed_at": "2026-01-24T10:30:00Z"
    }
  ]
}
EOF
pass "Created agent state for feature-1"

# Test 5: List agents
echo -e "\n${BLUE}Test 5: List agents${NC}"
cd "$TEST_DIR"
if $WTA list 2>&1 | grep -q "feature-1"; then
    pass "wta list shows the agent"
else
    fail "wta list should show the agent"
fi

# Test 6: Check current branch before merge
echo -e "\n${BLUE}Test 6: Verify we're on main before merge${NC}"
cd "$TEST_DIR"
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" = "main" ]; then
    pass "Currently on main branch"
else
    fail "Should be on main branch, but on $CURRENT_BRANCH"
fi

# Test 7: Test merge without --target (should merge to main)
echo -e "\n${BLUE}Test 7: Merge to default branch (main)${NC}"
cd "$TEST_DIR"
info "Running: wta merge 1"

# Capture output
MERGE_OUTPUT=$($WTA merge 1 2>&1)
echo "$MERGE_OUTPUT"

# Check if merge was successful
if echo "$MERGE_OUTPUT" | grep -q "Successfully merged"; then
    pass "Merge completed successfully"
else
    fail "Merge should succeed"
fi

# Test 8: Verify we're still on main after merge
echo -e "\n${BLUE}Test 8: Verify branch after merge${NC}"
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" = "main" ]; then
    pass "Still on main branch after merge"
else
    fail "Should be on main branch after merge, but on $CURRENT_BRANCH"
fi

# Test 9: Verify feature1.txt is now in main
echo -e "\n${BLUE}Test 9: Verify merged content${NC}"
if [ -f "$TEST_DIR/feature1.txt" ]; then
    CONTENT=$(cat "$TEST_DIR/feature1.txt")
    if [ "$CONTENT" = "Feature 1 code" ]; then
        pass "Merged content is present in main branch"
    else
        fail "Merged content has wrong value"
    fi
else
    fail "feature1.txt should exist in main after merge"
fi

# Test 10: Verify worktree was removed
echo -e "\n${BLUE}Test 10: Verify worktree cleanup${NC}"
if [ -d "$TEST_DIR-wta-1" ]; then
    fail "Worktree should be removed after merge"
else
    pass "Worktree was removed after merge"
fi

# Test 11: Test --target flag with explicit branch
echo -e "\n${BLUE}Test 11: Test --target flag${NC}"
git checkout main
git checkout -b develop
echo "Develop branch" > develop.txt
git add develop.txt
git commit -m "Add develop branch"

git checkout -b feature-2
echo "Feature 2 code" > feature2.txt
git add feature2.txt
git commit -m "Add feature 2"

# Go back to main before creating worktree
git checkout main
git worktree add "$TEST_DIR-wta-2" feature-2

# Add agent 2 to state
cat > "$TEST_DIR/.worktree-agents/state.json" << EOF
{
  "next_id": 3,
  "agents": [
    {
      "id": "2",
      "task": "Add feature 2",
      "branch": "feature-2",
      "base_branch": "develop",
      "worktree_path": "$TEST_DIR-wta-2",
      "tmux_session": "wta-test",
      "tmux_window": "2",
      "status": "completed",
      "provider": "claude",
      "launched_at": "2026-01-24T11:00:00Z",
      "completed_at": "2026-01-24T11:30:00Z"
    }
  ]
}
EOF

git checkout main
info "Running: wta merge 2 --target develop"
MERGE_OUTPUT=$($WTA merge 2 --target develop 2>&1)
echo "$MERGE_OUTPUT"

if echo "$MERGE_OUTPUT" | grep -q "Successfully merged"; then
    pass "Merge with --target completed successfully"
else
    fail "Merge with --target should succeed"
fi

# Verify we're on develop branch
git checkout develop
if [ -f "feature2.txt" ]; then
    pass "feature2.txt merged into develop branch (--target worked)"
else
    fail "feature2.txt should be in develop branch"
fi

# Test 12: Test running from worktree
echo -e "\n${BLUE}Test 12: Test commands from worktree${NC}"
git checkout main
git checkout -b feature-3
echo "Feature 3 code" > feature3.txt
git add feature3.txt
git commit -m "Add feature 3"

# Go back to main before creating worktree
git checkout main
git worktree add "$TEST_DIR-wta-3" feature-3

# Add to state
cat > "$TEST_DIR/.worktree-agents/state.json" << EOF
{
  "next_id": 4,
  "agents": [
    {
      "id": "3",
      "task": "Add feature 3",
      "branch": "feature-3",
      "base_branch": "main",
      "worktree_path": "$TEST_DIR-wta-3",
      "tmux_session": "wta-test",
      "tmux_window": "3",
      "status": "completed",
      "provider": "claude",
      "launched_at": "2026-01-24T12:00:00Z",
      "completed_at": "2026-01-24T12:30:00Z"
    }
  ]
}
EOF

# CD into the worktree and run commands
cd "$TEST_DIR-wta-3"
info "Running wta list from inside worktree"
if $WTA list 2>&1 | grep -q "feature-3"; then
    pass "wta list works from inside worktree"
else
    fail "wta list should work from inside worktree"
fi

# Try to merge from inside worktree (should still work)
cd "$TEST_DIR-wta-3"
info "Running wta merge 3 from inside worktree"
MERGE_OUTPUT=$($WTA merge 3 2>&1)

if echo "$MERGE_OUTPUT" | grep -q "Successfully merged"; then
    pass "wta merge works from inside worktree"
else
    echo "$MERGE_OUTPUT"
    fail "wta merge should work from inside worktree"
fi

# Verify merge went to main
cd "$TEST_DIR"
git checkout main
if [ -f "feature3.txt" ]; then
    pass "feature3.txt merged into main (commands work from worktree)"
else
    fail "feature3.txt should be in main"
fi

# Test 13: Verify git log
echo -e "\n${BLUE}Test 13: Verify git history${NC}"
cd "$TEST_DIR"
git checkout main
info "Git log on main branch:"
git log --oneline --all --graph | head -10

# Check that feature-1 and feature-3 are in main (feature-2 went to develop)
if git log --oneline | grep -q "Add feature 1"; then
    pass "Feature 1 is in main branch history"
else
    fail "Feature 1 should be in main"
fi

if git log --oneline | grep -q "Add feature 3"; then
    pass "Feature 3 is in main branch history"
else
    fail "Feature 3 should be in main"
fi

# Summary
echo -e "\n${GREEN}=== ALL TESTS PASSED ===${NC}"
echo -e "\nTest Summary:"
echo "  ✓ Default branch detection"
echo "  ✓ Merge to detected default branch"
echo "  ✓ --target flag for explicit branch"
echo "  ✓ Commands work from worktrees"
echo "  ✓ Worktree cleanup after merge"
echo "  ✓ Content verification"
echo "  ✓ Git history integrity"

echo -e "\n${GREEN}All merge functionality tests passed successfully!${NC}"
