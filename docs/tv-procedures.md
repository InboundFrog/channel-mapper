# On the TV

Steps and behaviour seen on the UE40MU6400 (UK model, firmware
T-KTMDEUC-1410.1). Menu names may differ slightly on other models.

## Export

1. Plug in a **FAT32** USB stick (ChanSort's documentation recommends
   FAT32, 32 GB or smaller).
2. **Settings → Broadcasting → Expert Settings → Transfer Channel List**.
3. Enter the PIN: `0000` unless it's been changed.
4. **Export to USB**. This writes `Channel_list_<firmware>.zip`, e.g.
   `Channel_list_T-KTMDEUC-1410.1.zip`.

Keep the original export somewhere safe. If an import goes wrong, you can
re-import it, or run a rescan.

## Import

1. Put tvmap's output on the stick under the **same file name** as the
   export, replacing the original on the stick (but keep a copy).
2. **Transfer Channel List → Import from USB**.
3. **The TV reboots.** The new numbers are there when it comes back.

## Afterwards

- **Turn off stand-by auto-tune.** Otherwise the TV can rescan while in
  stand-by and put everything back in the 800s.
- **After any rescan or firmware update**, export again and re-apply your
  saved map: `tvmap apply <zip> --map tvmap-out/channel-map.toml`. Don't
  import a list exported from an older firmware; the schema may differ.

## Firmware

- The export's file name shows the firmware version, which is easier than
  finding it in the menus.
- If you plan a firmware update, update first and then export, so you work
  on the file from the firmware the TV will be running. Use firmware for
  the exact model code on the TV's label, including the region suffix
  (UK sets end in `UXXU`).

## Typing channel numbers

After a digit, the TV waits before changing channel, in case more digits
follow (7 could be the start of 72 or 700). We didn't find a setting to
shorten the wait.

To change straight away, type the number and press **OK** (the centre
button). Samsung's manuals describe entering the number and then pressing
Select. For someone who finds an extra step hard to remember, just waiting
works too: the TV gets to the right channel a moment later.

## Other menu notes

- **Edit Channels → Change Number** exists, but is commonly greyed out
  for digital channels on Samsungs (see the SamyGO thread linked in
  [background.md](background.md)).
- This model has **no Country (Area)** option under Broadcasting.
