#!/usr/bin/env bash
# BLOCKING: empêche commits sur main/master et actions destructrices.
# Exit 2 bloque l'action + stderr est retourné à Claude.

set -uo pipefail

TOOL_INPUT="${CLAUDE_TOOL_INPUT:-}"

# 1. Bloquer commits sur main/master
if echo "$TOOL_INPUT" | grep -qE '\bgit\s+commit\b'; then
  BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
  if [[ "$BRANCH" == "main" || "$BRANCH" == "master" ]]; then
    echo "❌ BLOCKED: Direct commits on '$BRANCH' are forbidden." >&2
    echo "   Create a feature branch: git checkout -b feature/your-change" >&2
    exit 2
  fi
fi

# 2. Bloquer force-push sur main/master
if echo "$TOOL_INPUT" | grep -qE 'git\s+push.*(-f|--force).*\b(main|master)\b'; then
  echo "❌ BLOCKED: Force-push on main/master is forbidden." >&2
  exit 2
fi

# 3. Bloquer DROP DATABASE / TRUNCATE sur prod
if echo "$TOOL_INPUT" | grep -qiE '(drop\s+database|truncate\s+table.*production)'; then
  echo "❌ BLOCKED: Destructive DB operation detected. Confirm manually." >&2
  exit 2
fi

# 4. Avertir (non-bloquant) si suppression de Cargo.lock
if echo "$TOOL_INPUT" | grep -qE 'rm\s+.*Cargo\.lock'; then
  echo "⚠️  WARNING: About to delete Cargo.lock. Sure?" >&2
fi

exit 0
