#!/bin/sh
# Idempotently populate a running Flight Review service from a pinned manifest.
set -eu

MANIFEST="${SEED_MANIFEST:-/usr/share/flight-review/seed/logs.json}"
API_URL="${SEED_API_URL:-http://127.0.0.1:${PORT:-8080}}"
WORK_DIR="${SEED_WORK_DIR:-/tmp/flight-review-seed}"
PUBLIC="${SEED_LOGS_PUBLIC:-true}"
AI_MODEL="${SEED_AI_MODEL:-}"

log() {
  printf '%s %s\n' '[seed]' "$*"
}

upload_log() {
  if [ -n "$AI_MODEL" ]; then
    curl -sS -o "$response" -w '%{http_code}' -X POST "$API_URL/api/upload" \
      -F "file=@$input;filename=$filename" \
      -F "is_public=$PUBLIC" \
      -F "description=$description" \
      -F "vehicle_name=$vehicle seed example" \
      -F "rating=$rating" \
      -F "tags=seed,px4-public-example" \
      -F "source=seed:flight-review-v1:$source_id" \
      -F "ai_model=$AI_MODEL"
  else
    curl -sS -o "$response" -w '%{http_code}' -X POST "$API_URL/api/upload" \
      -F "file=@$input;filename=$filename" \
      -F "is_public=$PUBLIC" \
      -F "description=$description" \
      -F "vehicle_name=$vehicle seed example" \
      -F "rating=$rating" \
      -F "tags=seed,px4-public-example" \
      -F "source=seed:flight-review-v1:$source_id"
  fi
}

for command in curl jq sha256sum; do
  command -v "$command" >/dev/null 2>&1 || {
    log "required command is unavailable: $command"
    exit 1
  }
done

test -f "$MANIFEST" || {
  log "manifest not found: $MANIFEST"
  exit 1
}

mkdir -p "$WORK_DIR"
trap 'rm -rf "$WORK_DIR"' EXIT INT TERM

ready=false
attempt=0
while [ "$attempt" -lt 60 ]; do
  if curl -fsS "$API_URL/health" >/dev/null 2>&1; then
    ready=true
    break
  fi
  attempt=$((attempt + 1))
  sleep 1
done

if [ "$ready" != true ]; then
  log "server did not become ready at $API_URL"
  exit 1
fi

total=$(jq -r '.logs | length' "$MANIFEST")
seeded=0
skipped=0
failed=0

log "checking $total pinned ULogs"

entries="$WORK_DIR/entries.jsonl"
jq -c '.logs[]' "$MANIFEST" > "$entries"
while IFS= read -r entry; do
  source_id=$(printf '%s' "$entry" | jq -r '.id')
  url=$(printf '%s' "$entry" | jq -r '.url')
  expected_sha=$(printf '%s' "$entry" | jq -r '.sha256')
  vehicle=$(printf '%s' "$entry" | jq -r '.vehicle')
  rating=$(printf '%s' "$entry" | jq -r '.rating')
  description=$(printf '%s' "$entry" | jq -r '.description')
  filename="${source_id}.ulg"

  if curl -fsS "$API_URL/api/logs?include_private=true&limit=10&search=$source_id" \
    | jq -e --arg filename "$filename" '.logs | any(.filename == $filename)' >/dev/null; then
    log "already present: $filename"
    skipped=$((skipped + 1))
    continue
  fi

  input="$WORK_DIR/$filename"
  response="$WORK_DIR/$source_id.json"
  log "downloading $filename ($vehicle)"
  if ! curl -fL --retry 3 --retry-delay 2 --compressed -o "$input" "$url"; then
    log "download failed: $filename"
    failed=$((failed + 1))
    continue
  fi

  actual_sha=$(sha256sum "$input" | awk '{print $1}')
  if [ "$actual_sha" != "$expected_sha" ]; then
    log "checksum mismatch: $filename"
    rm -f "$input"
    failed=$((failed + 1))
    continue
  fi

  if ! http_code=$(upload_log); then
    rm -f "$input"
    failed=$((failed + 1))
    log "upload request failed: $filename"
    continue
  fi
  rm -f "$input"

  case "$http_code" in
    2??)
      seeded=$((seeded + 1))
      log "seeded: $filename"
      ;;
    *)
      failed=$((failed + 1))
      message=$(jq -r '.error // .message // "upload failed"' "$response" 2>/dev/null || true)
      log "upload failed ($http_code): $filename: $message"
      ;;
  esac
done < "$entries"

log "seed check complete: imported=$seeded skipped=$skipped failed=$failed"
