#!/usr/bin/env bash
set -euo pipefail
FLOWGATE="${FLOWGATE:-mcp-flowgate}"
exec "$FLOWGATE" check --config "$(dirname "$0")/../frontrails.yaml"
