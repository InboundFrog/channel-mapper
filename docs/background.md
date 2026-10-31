# Background

## The problem

A UK-market Samsung UE40MU6400 (2017 M-series, Tizen) brought to Perth.
It tunes every Perth channel, but numbers them 800 to 857 instead of the
numbers in the TV guide (7, 72, 9, 90, 2, …). For someone who goes by the
printed guide, typing 7 and getting nothing is a real problem, not a
cosmetic one.

## Why the 800s

Australian broadcasters send a logical channel number (LCN) for each
service, and Australian TVs use it as the channel number. This TV doesn't:

- **The LCN isn't stored.** `SRV_DVB.lcn` (and `hdLcn`) is `65535` for all
  58 services in the export, so the firmware discarded the numbers rather
  than storing them and choosing not to use them. Copying `lcn` into the
  channel number therefore wasn't an option.
- **The TV still thinks it's in the UK.** The export's `metadata.xml` has
  `DIGITAL_COUNTRY` = `4670034`, which is `0x474252`, ASCII `GBR`. The
  Broadcasting menu on this model has no Country (Area) setting to change it.
- **The overflow order is alphabetical.** Services were numbered from 800
  in case-insensitive name order: `10`, `10 comedy`, `10 drama`, `10 HD`, …,
  `7 Perth`, …, `ABC Classic`, …, `you.tv`.

*Not confirmed:* our guess is that the UK firmware only accepts LCNs sent
the UK way. Both countries use a logical channel descriptor, but the UK
version is tied to UK-specific signalling. Services without a number it
accepts go to an overflow range starting at 800. The observations above fit
this, but we haven't decoded the broadcast to prove it.

## Reception is fine: VHF works

Before looking at the export, we assumed the TV was UHF only, because UK
TVs were said to be. It isn't. The `CHNL` table records the frequency each
multiplex was found on, and all five Perth multiplexes are on VHF Band III:

| Multiplex | Frequency | Channel |
|---|---|---|
| Seven | 177.5 MHz | VHF 6 |
| SBS | 184.5 MHz | VHF 7 |
| Nine | 191.5 MHz | VHF 8 |
| Ten | 219.5 MHz | VHF 11 |
| ABC | 226.5 MHz | VHF 12 |

The TV can only have read the service names by tuning those frequencies.

The "UK TVs are UHF only" advice was true of **analogue** sets, and UK
digital TV (Freeview) is still broadcast on UHF only. A modern digital TV
has one tuner chip that covers roughly 40 to 870 MHz, VHF and UHF
included. Whether the firmware scans VHF is a separate question. Some
European countries broadcast DVB-T on VHF Band III with 7 MHz channels, as
Australia does, so pan-European Samsung firmware includes it.

## Fixes considered

| Option | Outcome |
|---|---|
| Change the TV's country to Australia, then rescan | The proper fix, but this model's Broadcasting menu has no Country (Area) option |
| Change the region in the hidden service menu | Possible on some Samsungs, but that menu has settings that can break the TV. Kept as a last resort, not tried |
| **Edit Channels → Change Number** on the TV | Commonly greyed out for digital channels. Samsung forum advice is to export the list and use ChanSort |
| **ChanSort** (Windows app that edits exported lists) | Supports this file format (Samsung J/K/M/MU/Q `.zip`). Windows only, and a one-off edit: every rescan means redoing it by hand |
| Favourites list | Lets you reorder, but not renumber. Doesn't help someone typing numbers from a guide |
| Australian set-top box (around A$40–60) | Correct numbers without any editing, and survives rescans. Worth considering, but it adds a second remote |
| **tvmap** | Edits the exported list directly. The number for each channel lives in a map keyed on broadcast IDs, so after a rescan you re-apply the map instead of starting again |

## Sources

- [ChanSort](https://github.com/PredatH0r/ChanSort) source, Samsung loader
  (`source/ChanSort.Loader.Samsung/Zip/DbSerializer.cs`, `DbChannel.cs`)
- [ChanSort Samsung wiki page](https://github.com/PredatH0r/ChanSort/wiki/Samsung):
  J, K, M, MU and Q series export `.zip` lists
- [SamyGO forum: renumbering digital channels](https://forum.samygo.tv/viewtopic.php?p=87049):
  Change Number greyed out; advice to use ChanSort instead
