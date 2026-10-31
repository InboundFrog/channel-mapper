# On the TV

Steps and behaviour seen on the UE40MU6400 (UK model, firmware
T-KTMDEUC-1410.1). Menu names may differ slightly on other models.

## Export

1. Plug in a **FAT32** USB stick. ChanSort's README asks for a stick of
   32 GB or less with FAT32.
2. **Settings → Broadcasting → Expert Settings → Transfer Channel List**.
3. Enter the PIN: `0000` unless it's been changed. Samsung's manuals show
   the PIN prompt for this menu, and ChanSort's Samsung page gives `0000`.
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

- **Turn off stand-by auto-tune.** We did this after importing. Otherwise
  the TV could rescan while in stand-by and put everything back in the
  800s. We haven't seen it happen; it's a precaution.
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
button). Samsung's e-Manual for a 2021 set describes entering the number
and then selecting, or waiting for the TV to change after a moment. Other
brands work the same way: Sony's support pages give the same advice to
press the centre Enter button after the number. For someone who finds an extra step hard to remember, just waiting
works too: the TV gets to the right channel a moment later.

## Other menu notes

- **Edit Channels → Change Number** exists, but is commonly greyed out
  for digital channels on Samsungs (see the SamyGO thread linked in
  [background.md](background.md)).
- This model has **no Country (Area)** option under Broadcasting.

## Sources

All read 31 October 2026. See also [sources.md](sources.md).

- ChanSort [README](https://github.com/PredatH0r/ChanSort) (USB stick: FAT32,
  32 GB or less) and [Samsung wiki page](https://github.com/PredatH0r/ChanSort/wiki/Samsung)
  (Transfer Channel List, PIN `0000`)
- Samsung TV e-Manual,
  [Transfer Channel List](https://www.manualslib.com/manual/816924/Samsung-Tv.html?page=23)
  (PIN prompt, Import from / Export to USB)
- Samsung QN92A e-Manual,
  [entering channel numbers](https://www.manualslib.com/manual/3392312/Samsung-Qn92a.html?page=239)
- Sony, [faster channel switching on BRAVIA 3](https://www.sony.ie/electronics/support/televisions-projectors/articles/00358103)
  (press Enter after the number)
- [SamyGO forum](https://forum.samygo.tv/viewtopic.php?p=87049)
  (Change Number greyed out for digital channels)
- What we saw on the UE40MU6400: menu paths, the reboot on import, no
  Country (Area) option
