#!/usr/bin/env bash
# Run a command. If it fails, repeat the end of its output as a GitHub
# annotation, so the failure can be read through the API without the log
# download.
log=$(mktemp)
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  body=$(grep -v -E '^\s*(Compiling|Checking|Downloaded|Downloading|Updating|Locking|Adding|Fresh|Blocking) ' "$log" | tail -n 150)
  body=${body//'%'/'%25'}
  body=${body//$'\r'/'%0D'}
  body=${body//$'\n'/'%0A'}
  echo "::error title=$* failed::$body"
fi
exit "$status"
