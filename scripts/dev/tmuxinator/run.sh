#!/usr/bin/env bash

set -euo pipefail


source scripts/_common.sh

ensure_in_dev_shell
build_workspace
add_target_dir_to_path


# if we are running, in a mode where we already switched from a previous
# tmux session, switch back
if [ -n "${FM_TMUXINATOR_SESSION:-}" ]; then
  >&2 echo "Already running in tmuxinator session"
  >&2 echo "Kill it with: 'tmux kill-session' and try again"
  if [ -n "${FM_TMUXINATOR_BASE_SESSION}" ]; then
    >&2 echo "Or switch to the base one with: 'tmux switch-session -t ${FM_TMUXINATOR_BASE_SESSION}'"
    exit 1
  fi
fi

if [ -n "$TMUX" ]; then
  # if already running in tmux, use the current session name as a base
  base_session="$(tmux display-message -p '#{session_name}')"
  session="${base_session}-devimint"
else
  # not in tmux, use cwd as a session identifier, so different git worktrees / checkouts
  # can run separate devimint sessions and be distinguishable from each other
  dir="${1:-${PWD}}"
  rel_pwd="${dir//${HOME}/\~${USER}}"
  session="${rel_pwd//./_}"
  session="${session}-devimint"
fi

export FM_TMUXINATOR_BASE_SESSION="${base_session}"
export FM_TMUXINATOR_SESSION="${session}"

if tmux -L fedimint-dev has-session -t "$session" 2>/dev/null; then
   >&2 echo "Killing previous tmux session ${session}"
   tmux -L fedimint-dev kill-session -t "$session"
fi

function run_tmuxinator {
  set -euo pipefail

  session="$FM_TMUXINATOR_SESSION"
  tmuxinator start local -n "${session}"

  tmux -L fedimint-dev attach-session -t "$session"
  tmux -L fedimint-dev kill-session -t "$session"
}
export -f run_tmuxinator

SHELL=$(which bash) devimint "$@" --link-test-dir "${CARGO_BUILD_TARGET_DIR:-$PWD/target}/devimint" dev-fed --exec bash -c run_tmuxinator
