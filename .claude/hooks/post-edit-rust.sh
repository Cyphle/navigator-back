#!/usr/bin/env bash
# Non-blocking: formate et lint après chaque édition.
# Exit 0 toujours pour ne pas interrompre le raisonnement de Claude.

set -uo pipefail

FILE="${CLAUDE_FILE_PATH:-}"
[ -z "$FILE" ] && exit 0
[ ! -f "$FILE" ] && exit 0

case "$FILE" in
  *.rs)
    rustfmt --edition 2021 "$FILE" 2>/dev/null || true
    if command -v cargo >/dev/null 2>&1; then
      cargo clippy --no-deps --message-format=short --quiet 2>&1 \
        | grep -E "(warning|error)" \
        | head -30 || true
    fi
    ;;
  *.toml)
    if [[ "$FILE" == *"Cargo.toml" ]] && command -v cargo >/dev/null 2>&1; then
      cargo verify-project --manifest-path "$FILE" 2>&1 | head -5 || true
    fi
    ;;
  *.sql)
    # Placeholder: ajoute sqlfluff ou pg_format si utilisé
    # sqlfluff fix "$FILE" --dialect postgres 2>/dev/null || true
    ;;
esac

exit 0
