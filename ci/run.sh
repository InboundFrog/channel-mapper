#!/usr/bin/env bash
# Run a command. If it fails, repeat its diagnostics as a GitHub annotation,
# so the failure can be read through the API without downloading the log.
export CARGO_TERM_COLOR=never
log=$(mktemp)
"$@" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  clean=$(sed -E 's/\x1b\[[0-9;]*[A-Za-z]//g' "$log" |
    grep -v -E '^\s*(Compiling|Checking|Downloaded|Downloading|Updating|Locking|Adding|Fresh|Blocking|Running|Finished) ')
  # Prefer everything from the first compiler diagnostic onwards.
  diag=$(printf '%s\n' "$clean" | sed -n -E '/^(error|warning)/,$p')
  body=$(printf '%s\n' "${diag:-$clean}" | head -n 300)
  body=${body//'%'/'%25'}
  body=${body//$'\r'/'%0D'}
  body=${body//$'\n'/'%0A'}
  echo "::error title=$* failed::$body"
fi
exit "$status"
