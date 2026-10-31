# tvmap

Renumber channels in a Samsung TV channel-list export (`Channel_list_*.zip`).

Written for a UK Samsung (UE40MU6400) used in Australia. UK firmware ignores
Australian logical channel numbers, so every channel lands at 800+. tvmap
puts them back on the numbers in the TV guide (7, 72, 9, 90, 2, 20, …).

It should work on other Samsung Tizen sets (2015+, J/K/M/N/Q series) that
export a `.zip` channel list.

**NOTE:** The purpose was for a family member with Alzheimers, using a UK
          TV. The UK firmware stripped Australian channel numbers. Getting
          someone with Alzheimers to remember new channels that don't match
          the TV guide is not an option.
          May work for other TVs, but it's not a priority for me.

## Use

1. On the TV: **Settings → Broadcasting → Expert Settings → Transfer Channel
   List**, PIN `0000`, **Export to USB**. Use a FAT32 stick.
2. Run tvmap on the exported zip:

   ```sh
   cargo build --release
   ./target/release/tvmap edit /Volumes/USB/Channel_list_T-KTMDEUC-1410.1.zip
   ```

   This opens the editor with the built-in Perth map applied. Press `w` to
   write. The new list goes to `tvmap-out/` under the **same file name**
   (the TV looks for it), and the map you ended up with goes to
   `tvmap-out/channel-map.toml`.
3. Copy `tvmap-out/Channel_list_….zip` onto the stick, replacing the export
   (keep a copy of the original), then **Import from USB** on the TV.

After a rescan, export again and re-apply your saved map:

```sh
tvmap apply Channel_list_….zip --map tvmap-out/channel-map.toml
```

### Commands

| Command | |
|---|---|
| `tvmap list ZIP` | Show every service and what the map would do |
| `tvmap apply ZIP [-o OUT] [--dry-run] [--save-map FILE]` | Apply without the editor |
| `tvmap edit ZIP [-o OUT] [--save-map FILE]` | Interactive editor |

All take `--map FILE` (your own map) or `--empty` (start from nothing)
instead of the built-in Perth map.

### Editor keys

| Key | |
|---|---|
| `1`–`9`, `Enter`, `e` | Type a channel number |
| `k` | Keep the TV's own number (e.g. leave it in the 800s) |
| `h` | Toggle hidden (removed from the channel list) |
| `s` | Toggle skip (typing the number works, CH+/CH− skips it) |
| `o` | Accept a flagged match as OK |
| `r` | Revert the row to what the map proposed |
| `n` | Jump to the next row that needs a look |
| `a` | Show only rows that need a look |
| `w` | Write the zip and map |
| `q` | Quit |

Yellow rows need a look, red rows share a number with another row. tvmap
won't write while any number is used twice.

## Tested on

| TV | Firmware | Region | Result |
|---|---|---|---|
| UE40MU6400 (UK model) | T-KTMDEUC-1410.1 | Perth, WA | Works. All 58 services renumbered. |

Notes from that TV:

- It reboots after **Import from USB**. That's normal; the new numbers are
  there when it comes back.
- Turn off **stand-by auto-tune** (Broadcasting settings) afterwards,
  otherwise the TV can rescan overnight and put everything back in the 800s.
- The UK firmware tunes Australian VHF channels fine (all five Perth
  multiplexes are on VHF 6–12). Only the numbering is wrong.

## How matching works

Each broadcast service is identified by original network ID, transport
stream ID and service ID (`onid/tsid/sid`). Those survive a rescan. For each
service, in order:

1. **IDs in the map**: use the mapped number. If the name is quite different
   from the one in the map, flag it (`name changed`).
2. **IDs not in the map, but exactly one map entry with the same name whose
   IDs are no longer broadcast**: use it, flagged (`matched by name`). That
   usually means the broadcaster changed a service ID.
3. **Broadcast LCN present**: use it. (UK firmware stores 65535 for
   Australian services, so this only helps on other firmware.)
4. **Otherwise**: leave it alone and flag it (`not in map`).

## What gets written

Only the `SRV` columns `major` (channel number), `hidden` + `hideGuide`, and
`numSel` (skip), and only where something changed. Names and IDs are never
written. Samsung stores names as byte-swapped UTF-16, and writing them back
is a known way to corrupt the list. Every other file in the zip is copied
byte-for-byte, and the original export is never modified.

The tests run against a real export (`tests/fixtures/`) and check exactly
that: after applying, the only differing column in the whole database is
the one meant to change.

## Credits

The file format notes come from [ChanSort](https://github.com/PredatH0r/ChanSort)'s
Samsung loader. Perth channel numbers are from Wikipedia's *List of digital
television channels in Australia* (October 2026).
