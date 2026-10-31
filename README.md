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
   ./target/release/tvmap edit /Volumes/USB/Channel_list_T-KTMDEUC-1410.1.zip --map maps/au/wa/perth.toml
   ```

   This opens the editor with the Perth map applied (see [Maps](#maps) for
   other cities). Press `w` to
   write. The new list goes to `tvmap-out/` under the **same file name**
   (the TV looks for it), and the map you ended up with goes to
   `tvmap-out/channel-map.toml`.
3. Copy `tvmap-out/Channel_list_….zip` onto the stick, replacing the export
   (keep a copy of the original), then **Import from USB** on the TV.

After a rescan, export again and re-apply your saved map. Without `--map`,
tvmap reads `tvmap-out/channel-map.toml`, so from the same folder this is
enough:

```sh
tvmap apply Channel_list_….zip
```

### Commands

| Command                                                  |                                              |
|----------------------------------------------------------|----------------------------------------------|
| `tvmap list ZIP`                                         | Show every service and what the map would do |
| `tvmap apply ZIP [-o OUT] [--dry-run] [--save-map FILE]` | Apply without the editor                     |
| `tvmap edit ZIP [-o OUT] [--save-map FILE]`              | Interactive editor                           |

All take `--map FILE` or `--empty` (start from nothing). With neither, the
map is read from `tvmap-out/channel-map.toml`; if that doesn't exist, tvmap
stops and asks for one. The map is read at run time, so editing it never
needs a rebuild.

### Maps

One map per city, under `maps/au/<state>/`:

| Map                  | Status                          |
|----------------------|---------------------------------|
| `wa/perth.toml`      | Tested on a real TV (see below) |
| `nsw/sydney.toml`    | **Untested**, best effort       |
| `act/canberra.toml`  | **Untested**, best effort       |
| `vic/melbourne.toml` | **Untested**, best effort       |
| `sa/adelaide.toml`   | **Untested**, best effort       |
| `qld/brisbane.toml`  | **Untested**, best effort       |
| `tas/hobart.toml`    | **Untested**, best effort       |
| `nt/darwin.toml`     | **Untested**, best effort       |

The untested maps take their service IDs from Free TV Australia's DTTB
Service Information Register (Issue 16, June 2021), and number channels the
same way as Perth. Services that started after 2021 (ABC HD and NITV HD among
them) aren't in it, and IDs may have changed, so expect yellow rows: fix them
in the editor and press `w`, and the saved map is right for your TV from then
on. A tested map from another city is very welcome.

To regenerate them from a newer issue of the register, download the four
state sheets from Free TV's
[Technology & Innovation](https://www.freetv.com.au/what-we-do/technology-innovation/)
page and run (Python 3.11+, `pdftotext` from poppler):

```sh
tools/freetv_maps.py NSW-ACT.pdf QLD.pdf Vic-SA.pdf NT-WA-Tas.pdf
git diff maps/
```

URLs work too. `--check` writes nothing and exits non-zero if a map would
change. Once a city's map has been tested on a real TV, take it out of
`CITIES` in the script so it isn't overwritten.

### Editor keys

| Key                   |                                                         |
|-----------------------|---------------------------------------------------------|
| `1`–`9`, `Enter`, `e` | Type a channel number                                   |
| `k`                   | Keep the TV's own number (e.g. leave it in the 800s)    |
| `h`                   | Toggle hidden (removed from the channel list)           |
| `s`                   | Toggle skip (typing the number works, CH+/CH− skips it) |
| `o`                   | Accept a flagged match as OK                            |
| `r`                   | Revert the row to what the map proposed                 |
| `n`                   | Jump to the next row that needs a look                  |
| `a`                   | Show only rows that need a look                         |
| `w`                   | Write the zip and map                                   |
| `q`                   | Quit                                                    |

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
television channels in Australia* (October 2026). Service IDs for the other
cities are from Free TV Australia's
[DTTB Service Information Register](https://www.freetv.com.au/what-we-do/technology-innovation/)
(Issue 16, June 2021).
