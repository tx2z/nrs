#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# nrs Release Script
# ============================================================================
# Prepares and publishes a new release of nrs
#
# Features:
# - Runs verification checks (fmt, clippy, tests, build)
# - Validates git repository is clean
# - Generates changelog from conventional commits using git-cliff
# - Updates version in Cargo.toml
# - Creates git tag and pushes to remote
# - GitHub Actions handles cross-platform builds and crates.io publishing
#
# Usage: ./scripts/release.sh [options]
#
# Options:
#   --allow-uncommitted    Allow release with uncommitted git changes
#   --skip-verify          Skip fmt/clippy/tests checks
#   --dry-run              Preview without making changes
#   -h, --help             Show this help message
# ============================================================================

# ============================================================================
# Color Definitions
# ============================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ============================================================================
# Global Variables
# ============================================================================
BUMP_TYPE=""
NEW_VERSION=""
CURRENT_VERSION=""
ALLOW_UNCOMMITTED=false
SKIP_VERIFY=false
DRY_RUN=false
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ============================================================================
# Argument Parsing
# ============================================================================
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --allow-uncommitted)
            ALLOW_UNCOMMITTED=true
            shift
            ;;
        --skip-verify)
            SKIP_VERIFY=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h | --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --allow-uncommitted    Allow release with uncommitted git changes"
            echo "  --skip-verify          Skip fmt/clippy/tests checks"
            echo "  --dry-run              Preview without making changes"
            echo "  -h, --help             Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} Unknown option: $1"
            echo "Use --help for usage information."
            exit 1
            ;;
        esac
    done
}

# ============================================================================
# Output Helper Functions
# ============================================================================
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

step() {
    echo -e "\n${MAGENTA}${BOLD}==> $1${NC}"
}

divider() {
    echo -e "${CYAN}────────────────────────────────────────────────────────────${NC}"
}

# Prompt for y/N confirmation (single keypress, no Enter needed)
confirm_prompt() {
    local message="$1"
    echo -e -n "${YELLOW}${message} (y/N)${NC} "
    read -n 1 -r confirm
    echo "" # New line after single char input
    [[ "$confirm" =~ ^[Yy]$ ]]
}

# Parse command line arguments
parse_args "$@"

# ============================================================================
# Cleanup Trap
# ============================================================================
cleanup() {
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        echo ""
        error "Release script failed with exit code: $exit_code"

        # Check if we created uncommitted changes
        if [[ -n "$(git status --porcelain 2>/dev/null)" ]]; then
            warn "There are uncommitted changes in your working directory."
            warn "You may need to review and revert them:"
            echo "  git status"
            echo "  git checkout -- Cargo.toml Cargo.lock CHANGELOG.md"
        fi

        # Check if we created a local tag that wasn't pushed
        if [[ -n "${NEW_VERSION:-}" ]]; then
            local tag_name="v${NEW_VERSION}"
            if git rev-parse "$tag_name" &>/dev/null; then
                if ! git ls-remote --tags origin 2>/dev/null | grep -q "refs/tags/$tag_name"; then
                    warn "Local tag '$tag_name' was created but not pushed."
                    warn "To remove it: git tag -d $tag_name"
                fi
            fi
        fi
    fi
}

trap cleanup EXIT

# ============================================================================
# Pre-flight Checks
# ============================================================================
preflight_checks() {
    step "Running pre-flight checks..."

    # Check we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        error "Must be run from the project root directory (Cargo.toml not found)"
        exit 1
    fi

    # Check required tools are available
    local required_tools=("git" "cargo" "git-cliff")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &>/dev/null; then
            error "Required tool not found: $tool"
            if [[ "$tool" == "git-cliff" ]]; then
                echo "  Install with: cargo install git-cliff"
            fi
            exit 1
        fi
    done

    success "All required tools are available"
}

# ============================================================================
# Step 1: Check Git Repository Status
# ============================================================================
check_git_status() {
    step "Step 1: Checking git repository status..."
    divider

    cd "$PROJECT_ROOT"

    # Check for uncommitted changes (staged or unstaged)
    if [[ -n "$(git status --porcelain)" ]]; then
        if [[ "$ALLOW_UNCOMMITTED" == true ]]; then
            warn "Git repository has uncommitted changes:"
            echo ""
            git status --short
            echo ""
            warn "These files will NOT be included in the release."
            if ! confirm_prompt "Continue with release anyway?"; then
                info "Release cancelled."
                exit 0
            fi
        else
            error "Git repository has uncommitted changes!"
            echo ""
            git status --short
            echo ""
            error "Please commit or stash your changes before releasing."
            echo "  Or use --allow-uncommitted to skip this check."
            exit 1
        fi
    fi

    # Check we're on the main branch
    local current_branch
    current_branch=$(git branch --show-current)

    if [[ "$current_branch" != "main" ]]; then
        warn "You are on branch '$current_branch', not 'main'"
        if ! confirm_prompt "Are you sure you want to release from this branch?"; then
            info "Release cancelled."
            exit 0
        fi
    fi

    # Check if we're up to date with remote
    git fetch origin "$current_branch" --quiet 2>/dev/null || true
    local local_commit remote_commit
    local_commit=$(git rev-parse HEAD)
    remote_commit=$(git rev-parse "origin/$current_branch" 2>/dev/null || echo "")

    if [[ -n "$remote_commit" ]] && [[ "$local_commit" != "$remote_commit" ]]; then
        warn "Local branch is not up to date with origin/$current_branch"
        if ! confirm_prompt "Continue anyway?"; then
            info "Release cancelled. Please pull/push changes first."
            exit 0
        fi
    fi

    success "Git repository is ready"
}

# ============================================================================
# Step 2: Run Verification Suite
# ============================================================================
run_verification() {
    if [[ "$SKIP_VERIFY" == true ]]; then
        warn "Skipping verification (--skip-verify flag)"
        return 0
    fi

    step "Step 2: Running verification suite..."
    divider

    cd "$PROJECT_ROOT"

    info "Checking formatting..."
    if ! cargo fmt --check; then
        error "Formatting check failed! Run 'cargo fmt' to fix."
        exit 1
    fi
    success "Formatting OK"

    info "Running clippy..."
    if ! cargo clippy -- -D warnings; then
        error "Clippy found warnings! Please fix them before releasing."
        exit 1
    fi
    success "Clippy OK"

    info "Running tests..."
    if ! cargo test; then
        error "Tests failed! Please fix them before releasing."
        exit 1
    fi
    success "Tests OK"

    info "Building release..."
    if ! cargo build --release; then
        error "Release build failed!"
        exit 1
    fi
    success "Release build OK"

    success "All verification checks passed!"
}

# ============================================================================
# Step 3: Get Last Tag and Show Changes
# ============================================================================
get_last_tag() {
    # Get the last version tag (format: v1.0.0)
    git describe --tags --abbrev=0 --match "v*" 2>/dev/null || echo ""
}

show_changes_since_last_tag() {
    step "Step 3: Analyzing changes since last release..."
    divider

    cd "$PROJECT_ROOT"

    local last_tag
    last_tag=$(get_last_tag)

    if [[ -z "$last_tag" ]]; then
        info "No previous version tags found. This will be the first release."
        echo ""
        info "Recent commits to be included:"
        git log --oneline --no-decorate | head -20
    else
        info "Last release: $last_tag"
        echo ""

        # Count commits since last tag
        local commit_count
        commit_count=$(git rev-list "$last_tag"..HEAD --count)

        if [[ "$commit_count" -eq 0 ]]; then
            warn "No new commits since $last_tag"
            if ! confirm_prompt "Are you sure you want to create a new release?"; then
                info "Release cancelled."
                exit 0
            fi
        else
            info "Changes since $last_tag ($commit_count commits):"
            echo ""
            git log "$last_tag"..HEAD --oneline --no-decorate
        fi
    fi

    echo ""
}

# ============================================================================
# Step 4: Prompt for Version Bump
# ============================================================================
get_current_version() {
    # Extract version from Cargo.toml
    grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

calculate_new_version() {
    local current_version="$1"
    local bump_type="$2"

    # Parse current version
    local major minor patch
    IFS='.' read -r major minor patch <<<"$current_version"

    # Default to 0 if parsing fails
    major=${major:-0}
    minor=${minor:-0}
    patch=${patch:-0}

    case "$bump_type" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch)
        patch=$((patch + 1))
        ;;
    *)
        error "Invalid bump type: $bump_type"
        exit 1
        ;;
    esac

    echo "${major}.${minor}.${patch}"
}

prompt_version_bump() {
    step "Step 4: Version bump selection..."
    divider

    CURRENT_VERSION=$(get_current_version)

    info "Current version: $CURRENT_VERSION"
    echo ""

    # Show what each bump would produce
    local patch_version minor_version major_version
    patch_version=$(calculate_new_version "$CURRENT_VERSION" "patch")
    minor_version=$(calculate_new_version "$CURRENT_VERSION" "minor")
    major_version=$(calculate_new_version "$CURRENT_VERSION" "major")

    echo "  Select version bump type:"
    echo ""
    echo -e "    ${CYAN}1)${NC} patch  -> ${GREEN}$patch_version${NC}  (bug fixes, small changes)"
    echo -e "    ${CYAN}2)${NC} minor  -> ${GREEN}$minor_version${NC}  (new features, backwards compatible)"
    echo -e "    ${CYAN}3)${NC} major  -> ${GREEN}$major_version${NC}  (breaking changes)"
    echo ""
    echo -n "  Enter choice (1/2/3): "
    read -r choice

    case "$choice" in
    1 | patch)
        BUMP_TYPE="patch"
        NEW_VERSION="$patch_version"
        ;;
    2 | minor)
        BUMP_TYPE="minor"
        NEW_VERSION="$minor_version"
        ;;
    3 | major)
        BUMP_TYPE="major"
        NEW_VERSION="$major_version"
        ;;
    *)
        error "Invalid choice: $choice"
        exit 1
        ;;
    esac

    echo ""
    success "Selected: $BUMP_TYPE bump -> v$NEW_VERSION"
}

# ============================================================================
# Step 5: Generate Changelog
# ============================================================================
generate_changelog() {
    step "Step 5: Generating CHANGELOG.md..."
    divider

    cd "$PROJECT_ROOT"

    if [[ "$DRY_RUN" == true ]]; then
        info "[DRY-RUN] Would generate changelog for v$NEW_VERSION"
        info "Preview of changes:"
        git-cliff --unreleased --tag "v$NEW_VERSION" 2>/dev/null || warn "git-cliff preview failed"
        return 0
    fi

    # Generate changelog with git-cliff
    git-cliff --tag "v$NEW_VERSION" -o CHANGELOG.md

    success "CHANGELOG.md generated"

    # Show preview
    info "Changelog preview (first 40 lines):"
    echo ""
    head -40 CHANGELOG.md
    echo ""
}

# ============================================================================
# Step 6: Update Cargo.toml Version
# ============================================================================
update_cargo_version() {
    step "Step 6: Updating Cargo.toml version..."
    divider

    cd "$PROJECT_ROOT"

    if [[ "$DRY_RUN" == true ]]; then
        info "[DRY-RUN] Would update version in Cargo.toml from $CURRENT_VERSION to $NEW_VERSION"
        return 0
    fi

    # Update version in Cargo.toml
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

    # Update Cargo.lock
    cargo check --quiet 2>/dev/null || true

    info "Updated Cargo.toml to v$NEW_VERSION"
    success "Version updated"
}

# ============================================================================
# Step 7: Git Commit, Tag, and Push
# ============================================================================
git_commit_tag_push() {
    step "Step 7: Creating git commit and tag..."
    divider

    cd "$PROJECT_ROOT"

    local tag_name="v${NEW_VERSION}"

    if [[ "$DRY_RUN" == true ]]; then
        info "[DRY-RUN] Would perform the following git operations:"
        echo "  - Stage: Cargo.toml Cargo.lock CHANGELOG.md"
        echo "  - Commit: chore(release): v${NEW_VERSION}"
        echo "  - Tag: $tag_name"
        echo "  - Push: commit and tag to origin"
        return 0
    fi

    # Stage the changed files
    git add Cargo.toml Cargo.lock CHANGELOG.md

    # Check if there are changes to commit
    if [[ -z "$(git diff --cached --name-only)" ]]; then
        warn "No changes to commit"
        return 0
    fi

    # Create commit
    git commit -m "chore(release): v${NEW_VERSION}

- Update version to ${NEW_VERSION}
- Generate changelog for release"

    info "Created commit for release v${NEW_VERSION}"

    # Create annotated tag
    git tag -a "$tag_name" -m "Release $tag_name"

    info "Created tag: $tag_name"

    # Confirm push
    echo ""
    echo -e "${YELLOW}Ready to push to remote. This will:${NC}"
    echo "  - Push commit to origin"
    echo "  - Push tag $tag_name to origin"
    echo "  - Trigger GitHub Actions to build and publish"
    echo ""
    if ! confirm_prompt "Continue?"; then
        warn "Push cancelled. Changes are committed locally."
        warn "Run the following manually when ready:"
        local current_branch
        current_branch=$(git branch --show-current)
        echo "  git push origin $current_branch"
        echo "  git push origin $tag_name"
        return 0
    fi

    # Push commit and tag
    local current_branch
    current_branch=$(git branch --show-current)

    git push origin "$current_branch"
    info "Pushed commit to origin/$current_branch"

    git push origin "$tag_name"
    info "Pushed tag $tag_name to origin"

    success "Git changes pushed successfully"
}

# ============================================================================
# Main Function
# ============================================================================
main() {
    echo ""
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo -e "${BOLD}${CYAN}              nrs Release Script                      ${NC}"
    echo -e "${BOLD}${CYAN}======================================================${NC}"
    echo ""

    # Show flag status
    if [[ "$DRY_RUN" == true ]]; then
        warn "Running in DRY-RUN mode (no changes will be made)"
    fi
    if [[ "$ALLOW_UNCOMMITTED" == true ]]; then
        warn "Running with --allow-uncommitted"
    fi
    if [[ "$SKIP_VERIFY" == true ]]; then
        warn "Running with --skip-verify"
    fi
    echo ""

    # Change to project root
    cd "$PROJECT_ROOT"

    # Run all steps
    preflight_checks
    check_git_status
    run_verification
    show_changes_since_last_tag
    prompt_version_bump
    generate_changelog
    update_cargo_version
    git_commit_tag_push

    # Final summary
    echo ""
    divider
    echo ""
    if [[ "$DRY_RUN" == true ]]; then
        echo -e "${YELLOW}${BOLD}Dry Run Complete!${NC}"
        echo ""
        echo "  No changes were made. Run without --dry-run to perform the release."
    else
        echo -e "${GREEN}${BOLD}Release Complete!${NC}"
        echo ""
        echo "  Version:    v${NEW_VERSION}"
        echo "  Tag:        v${NEW_VERSION}"
        echo ""
        echo "  GitHub Actions will now:"
        echo "    - Build binaries for macOS, Linux, and Windows"
        echo "    - Create a GitHub Release with the binaries"
        echo "    - Publish to crates.io"
        echo ""
        echo "  Monitor progress at: https://github.com/tx2z/nrs/actions"
    fi
    echo ""
    divider
}

# ============================================================================
# Script Entry Point
# ============================================================================
main "$@"
