#!/usr/bin/env bash
set -euo pipefail
FLOWGATE="${FLOWGATE:-mcp-flowgate}"
CONFIG="$(dirname "$0")/../frontrails.yaml"
# 1. Validate config (parse + resolve workflow/capability refs).
"$FLOWGATE" check --config "$CONFIG"
# 2. Fuzz every workflow with mock executors — catches wedges/livelocks/engine
#    errors in the burndown loop (and frontrails_spec / proxy_default). Exits
#    non-zero on any violation.
exec "$FLOWGATE" fuzz --config "$CONFIG"
