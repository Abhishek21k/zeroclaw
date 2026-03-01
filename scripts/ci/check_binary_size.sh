#!/usr/bin/env bash
# Check binary file size against safeguard thresholds.
#
# Usage: check_binary_size.sh <binary_path> [label]
#
# Arguments:
#   binary_path  Path to the binary to check (required)
#   label        Optional label for step summary (e.g. target triple)
#
# Thresholds:
#   hard limit     - BINARY_SIZE_HARD_LIMIT_BYTES (default: 20 MiB)
#   advisory limit - BINARY_SIZE_ADVISORY_LIMIT_BYTES (default: 15 MiB)
#   target limit   - BINARY_SIZE_TARGET_LIMIT_BYTES (default: 5 MiB)
#
# Writes to GITHUB_STEP_SUMMARY when the variable is set and label is provided.

set -euo pipefail

BIN="${1:?Usage: check_binary_size.sh <binary_path> [label]}"
LABEL="${2:-}"
HARD_LIMIT_BYTES="${BINARY_SIZE_HARD_LIMIT_BYTES:-20971520}"        # 20 MiB
ADVISORY_LIMIT_BYTES="${BINARY_SIZE_ADVISORY_LIMIT_BYTES:-15728640}" # 15 MiB
TARGET_LIMIT_BYTES="${BINARY_SIZE_TARGET_LIMIT_BYTES:-5242880}"      # 5 MiB

if [ ! -f "$BIN" ]; then
  echo "::error::Binary not found at $BIN"
  exit 1
fi

# macOS stat uses -f%z, Linux stat uses -c%s
SIZE=$(stat -f%z "$BIN" 2>/dev/null || stat -c%s "$BIN")
SIZE_MB=$((SIZE / 1024 / 1024))
SIZE_MIB=$(awk "BEGIN {printf \"%.2f\", ${SIZE}/1048576}")
HARD_LIMIT_MIB=$(awk "BEGIN {printf \"%.2f\", ${HARD_LIMIT_BYTES}/1048576}")
ADVISORY_LIMIT_MIB=$(awk "BEGIN {printf \"%.2f\", ${ADVISORY_LIMIT_BYTES}/1048576}")
TARGET_LIMIT_MIB=$(awk "BEGIN {printf \"%.2f\", ${TARGET_LIMIT_BYTES}/1048576}")
echo "Binary size: ${SIZE_MB}MB ($SIZE bytes)"

if [ -n "$LABEL" ] && [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  echo "### Binary Size: $LABEL" >> "$GITHUB_STEP_SUMMARY"
  echo "- Size: ${SIZE_MB}MB (${SIZE_MIB} MiB, $SIZE bytes)" >> "$GITHUB_STEP_SUMMARY"
fi

if [ "$SIZE" -gt "$HARD_LIMIT_BYTES" ]; then
  echo "::error::Binary exceeds hard safeguard (${HARD_LIMIT_MIB} MiB): ${SIZE_MIB} MiB ($SIZE bytes)"
  exit 1
elif [ "$SIZE" -gt "$ADVISORY_LIMIT_BYTES" ]; then
  echo "::warning::Binary exceeds advisory target (${ADVISORY_LIMIT_MIB} MiB): ${SIZE_MIB} MiB"
elif [ "$SIZE" -gt "$TARGET_LIMIT_BYTES" ]; then
  echo "::warning::Binary exceeds target (${TARGET_LIMIT_MIB} MiB): ${SIZE_MIB} MiB"
else
  echo "Binary size within target."
fi
