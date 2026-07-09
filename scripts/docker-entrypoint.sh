#!/bin/sh
set -eu

case "${1:-}" in
  serve|migrate|-*)
    set -- flight-review-server "$@"
    ;;
esac

case "${SEED_LOGS:-false}" in
  true|1|yes) seed_enabled=true ;;
  *) seed_enabled=false ;;
esac

if [ "$seed_enabled" != true ] || [ "${1:-}" != "flight-review-server" ] || [ "${2:-}" != "serve" ]; then
  exec "$@"
fi

"$@" &
server_pid=$!
seed_pid=''

stop_children() {
  if [ -n "$seed_pid" ]; then
    kill "$seed_pid" 2>/dev/null || true
  fi
  kill -TERM "$server_pid" 2>/dev/null || true
}
trap stop_children INT TERM

(
  if ! /usr/local/bin/seed-logs; then
    printf '%s\n' '[seed] seed pass failed; the server will stay available and retry on its next start' >&2
  fi
) &
seed_pid=$!

set +e
wait "$server_pid"
exit_code=$?
set -e

kill "$seed_pid" 2>/dev/null || true
wait "$seed_pid" 2>/dev/null || true
exit "$exit_code"
