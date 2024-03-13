#!/usr/bin/env bash

source ./scripts/dev/user-shell.sh

if [ -z "${FM_TMUXINATOR_SILENT:-}" ]; then
  echo ""
  echo "Important tmux key sequences:"
  echo ""
  echo "  ctrl+b <num>          - switching between panels (num: 1, 2, 3...)"
  echo "  ctrl+b :kill-session  - quit tmuxinator (or run 'tmux kill-session')"
fi
