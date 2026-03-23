#!/bin/zsh
# PR Triage Script — analyze all open PRs for conflicts, overlaps, and safe merge order
# Usage: ./scripts/pr-triage.sh [--auto-merge] [--close-conflicts]

set -euo pipefail

# Prerequisites check
for cmd in gh jq git; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: $cmd is required but not found" >&2; exit 1
  fi
done

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

AUTO_MERGE=false
CLOSE_CONFLICTS=false
for arg in "$@"; do
  case "$arg" in
    --auto-merge) AUTO_MERGE=true ;;
    --close-conflicts) CLOSE_CONFLICTS=true ;;
  esac
done

echo "${CYAN}PR Triage for amcli${NC}"
echo "================================"
echo ""

ORIG_BRANCH=$(git branch --show-current 2>/dev/null || echo "")
ORIG_HEAD=$(git rev-parse HEAD)

cleanup() {
  git merge --abort 2>/dev/null || true
  if [ -n "$ORIG_BRANCH" ]; then
    git checkout -q "$ORIG_BRANCH" 2>/dev/null || true
  else
    git checkout -q --detach "$ORIG_HEAD" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Fetch all remote branches in one call (avoids O(n^2) fetches)
git fetch origin '+refs/heads/*:refs/remotes/origin/*' --prune --quiet

# Get open PRs as JSON, then extract into parallel arrays
prs_json=$(gh pr list --base main --state open --json number,title,headRefName --limit 50)
pr_count=$(echo "$prs_json" | jq length)
echo "Found $pr_count open PRs"
echo ""

# Build parallel arrays from JSON
typeset -a pr_numbers pr_titles pr_refs
typeset -A files_by_pr  # zsh supports associative arrays natively

for i in $(seq 0 $((pr_count - 1))); do
  pr_numbers+=("$(echo "$prs_json" | jq -r ".[$i].number")")
  pr_titles+=("$(echo "$prs_json" | jq -r ".[$i].title" | head -c 60)")
  pr_refs+=("$(echo "$prs_json" | jq -r ".[$i].headRefName")")
done

mergeable_prs=()
conflict_prs=()

# Phase 1: Check each PR against current main
echo "${CYAN}Phase 1: Testing merge against main${NC}"
echo "------------------------------------"

for (( i=1; i<=pr_count; i++ )); do
  num=${pr_numbers[$i]}
  title=${pr_titles[$i]}
  ref=${pr_refs[$i]}

  # Get changed files for overlap detection later
  files=$(gh pr diff "$num" --name-only 2>/dev/null | sort | tr '\n' '|' || echo "")
  files_by_pr[$num]="$files"

  # Try merge into main (using already-fetched remote ref)
  git checkout -q --detach origin/main 2>/dev/null
  if git merge --no-commit --no-ff "origin/$ref" 2>/dev/null; then
    git merge --abort 2>/dev/null || git reset --hard origin/main 2>/dev/null
    echo "  ${GREEN}#${num}${NC} mergeable — $title"
    mergeable_prs+=("$num")
  else
    git merge --abort 2>/dev/null || git reset --hard origin/main 2>/dev/null || true
    echo "  ${RED}#${num}${NC} CONFLICT  — $title"
    conflict_prs+=("$num")
  fi
done

echo ""

# Phase 2: Detect overlapping PRs (same files modified)
echo "${CYAN}Phase 2: Detecting file overlaps${NC}"
echo "---------------------------------"

for (( i=1; i<=pr_count; i++ )); do
  num_i=${pr_numbers[$i]}
  files_i="${files_by_pr[$num_i]:-}"
  if [ -z "$files_i" ]; then continue; fi

  for (( j=i+1; j<=pr_count; j++ )); do
    num_j=${pr_numbers[$j]}
    files_j="${files_by_pr[$num_j]:-}"
    if [ -z "$files_j" ]; then continue; fi

    common=$(comm -12 \
      <(echo "$files_i" | tr '|' '\n' | grep -v '^$' | sort) \
      <(echo "$files_j" | tr '|' '\n' | grep -v '^$' | sort) \
      2>/dev/null || true)
    if [ -n "$common" ]; then
      file_list=$(echo "$common" | head -3 | tr '\n' ', ')
      echo "  ${YELLOW}#${num_i} <-> #${num_j}${NC} overlap on: ${file_list}"
    fi
  done
done

echo ""

# Phase 3: Sequential merge test — find safe merge order
echo "${CYAN}Phase 3: Finding safe merge order${NC}"
echo "----------------------------------"

safe_order=()
git checkout -q --detach origin/main 2>/dev/null

for num in "${mergeable_prs[@]}"; do
  # Find the ref for this PR number
  ref=""
  for (( i=1; i<=pr_count; i++ )); do
    if [ "${pr_numbers[$i]}" = "$num" ]; then
      ref=${pr_refs[$i]}
      break
    fi
  done
  if [ -z "$ref" ]; then continue; fi

  # Single merge operation — if it succeeds, we keep the commit for the next test
  if git merge --no-edit "origin/$ref" 2>/dev/null; then
    safe_order+=("$num")
    echo "  ${GREEN}Step ${#safe_order[@]}${NC}: merge #${num}"
  else
    git merge --abort 2>/dev/null || true
    echo "  ${YELLOW}Skip${NC}: #${num} conflicts after prior merges"
  fi
done

echo ""

# Summary
echo "================================"
echo "${CYAN}Summary${NC}"
echo "  Mergeable: ${#mergeable_prs[@]}"
echo "  Conflicts: ${#conflict_prs[@]}"
echo "  Safe sequential order: ${safe_order[*]:-none}"
echo ""

if [ ${#conflict_prs[@]} -gt 0 ]; then
  echo "${RED}Conflicting PRs (consider closing):${NC}"
  for pr in "${conflict_prs[@]}"; do
    title=$(echo "$prs_json" | jq -r ".[] | select(.number == $pr) | .title" | head -c 60)
    echo "  #$pr — $title"
  done
  echo ""
fi

# Auto-merge if requested
if [ "$AUTO_MERGE" = true ] && [ ${#safe_order[@]} -gt 0 ]; then
  echo "${YELLOW}Auto-merging ${#safe_order[@]} PRs in safe order...${NC}"
  echo "${YELLOW}Note: uses 'gh pr merge --squash' for immediate merge.${NC}"
  for num in "${safe_order[@]}"; do
    echo -n "  Merging #${num}... "
    if gh pr merge "$num" --squash 2>/dev/null; then
      echo "${GREEN}done${NC}"
      # Wait briefly for GitHub to process the merge
      sleep 2
    else
      echo "${RED}failed${NC}"
      echo "  Stopping — remaining PRs may need rebase"
      break
    fi
  done
fi

# Close conflicting PRs if requested
if [ "$CLOSE_CONFLICTS" = true ] && [ ${#conflict_prs[@]} -gt 0 ]; then
  echo "${YELLOW}Closing conflicting PRs...${NC}"
  for pr in "${conflict_prs[@]}"; do
    echo -n "  Closing #${pr}... "
    gh pr close "$pr" --comment "Closed by triage: conflicts with main after merging higher-priority PRs." 2>/dev/null && \
      echo "${GREEN}done${NC}" || echo "${RED}failed${NC}"
  done
fi
