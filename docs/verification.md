# Verification

The TV is someone's only TV, so the write path was checked several ways
before anything was imported.

## 1. Python prototype

Before the Rust tool existed, a short Python script (standard library only)
applied the Perth map to the real export and compared the result:

- Every row of every table, every column, read as raw bytes
  (`cast(col as blob)`), before and after.
- The only changes were 58 values in `SRV.major`.
- `typeof(srvName)` was still `text` for every row.
- `pragma integrity_check` returned `ok`.
- The other zip entries were identical.

That confirmed the approach (change `major`, `REINDEX`, re-zip) before
any Rust was written.

## 2. Tests against the real export

The export is checked in as `tests/fixtures/Channel_list_T-KTMDEUC-1410.1.zip`.
The tests in `tests/roundtrip.rs` cover:

| Test                                                                                                                  | Checks                                                                                  |
| --------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `reads_the_export`                                                                                                    | 58 services, all at 800–857, `lcn` 65535, names decode                                  |
| `perth_map_covers_everything`                                                                                         | Every service matches the Perth map with no flags and no conflicts                      |
| `main_numbers_are_hd`                                                                                                 | 2, 3, 7, 9 and 10 are HD services                                                       |
| `apply_changes_only_channel_numbers`                                                                                  | Full-database byte diff: only `SRV.major` changes. Integrity check ok, names still TEXT |
| `other_zip_entries_are_untouched`                                                                                     | Entry order kept; non-database entries byte-identical, compressed                       |
| `hide_and_skip_touch_only_their_columns`                                                                              | Hide changes `hidden` + `hideGuide`; skip changes `numSel`; nothing else                |
| `refuses_duplicate_numbers`                                                                                           | Won't write, and leaves no partial output                                               |
| `refuses_to_overwrite_the_original`                                                                                   | Original file unchanged                                                                 |
| `saved_map_reproduces_the_plan`                                                                                       | Save a map, reload it, get the same result                                              |
| `flags_renamed_services` / `falls_back_to_name_when_service_id_changes` / `ambiguous_name_fallback_is_left_unmatched` | Matching heuristics                                                                     |

The editor is tested in `src/tui.rs` with ratatui's `TestBackend`, by
feeding key presses and checking state and the rendered screen:
renumbering and re-sorting, conflicts blocking a write, hide/keep/skip, the
quit confirmation, and the problems-only filter.

## 3. CI on macOS and Linux

CI uses the shared workflows from
[repo-tools](https://github.com/InboundFrog/repo-tools), configured in
`.github/repo-tools.conf`. On `ubuntu-latest` and `macos-latest` it runs
`fmt`, `clippy -D warnings`, a release build, the tests, and a CLI smoke test
(`ci/smoke.sh`: list → apply → list with the saved map), then uploads the
binary and the smoke-test output. It also checks that the crate builds on its
declared minimum Rust version (1.88), runs `cargo audit`, and lints
`tools/freetv_maps.py` and the shell scripts.

If a step fails, the workflow strips colour codes and build-progress lines
and posts the diagnostics as a GitHub annotation. Annotations can be read
through the REST API (`/check-runs/{id}/annotations`) when the full job log
can't be downloaded, as was the case from the sandbox used to build this.

## 4. On the TV

The output was imported on 31 October 2026. The TV rebooted and came back
with every channel on its new number.
