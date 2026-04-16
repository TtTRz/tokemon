#!/bin/bash
# tokemon statusline bridge script
# Place this at ~/.claude/statusline.sh and configure in ~/.claude/settings.json:
#
#   {
#     "statusLine": {
#       "type": "command",
#       "command": "~/.claude/statusline.sh"
#     }
#   }
#
# This script receives JSON from Claude Code via stdin,
# forwards it to tokemon's Unix socket, and outputs a status line.

SOCKET="${TMPDIR:-/tmp}tokemon-claude.sock"

# Read JSON from stdin
data=$(cat)

# Forward to tokemon socket (non-blocking, ignore errors if tokemon isn't running)
if [ -S "$SOCKET" ]; then
    echo "$data" | nc -U -w0 "$SOCKET" 2>/dev/null &
fi

# Output status line for the terminal
model=$(echo "$data" | jq -r '.model.display_name // "?"')
ctx_pct=$(echo "$data" | jq -r '.context_window.used_percentage // 0' | xargs printf '%.0f')
cost=$(echo "$data" | jq -r '.cost.total_cost_usd // 0' | xargs printf '%.2f')

echo "${model} | ctx ${ctx_pct}% | \$${cost}"
