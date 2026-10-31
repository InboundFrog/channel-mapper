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

### How LCNs are sent, and why a UK set ignores Australia's

Both countries put the channel numbers in the same place: a
`logical_channel_descriptor` with tag `0x83`, carried in the Network
Information Table. The descriptor was designed by the UK's Digital TV Group
(DTG), which approved its use in Australia
([Free TV OP-41](https://www.freetv.com.au/wp-content/uploads/2024/02/Free-TV-OP-41-LCN-Descriptor-Issue-9-January-2024.pdf);
[DTTB Service Information Register](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-Introduction.pdf)).
The difference is how it's labelled:

|                             | UK (DTG D-Book)                                                 | Australia (Free TV OP-41)                                 |
| --------------------------- | --------------------------------------------------------------- | --------------------------------------------------------- |
| Descriptor tag              | `0x83`                                                          | `0x83`                                                    |
| Status                      | Private: must follow a `private_data_specifier` of `0x0000233A` | Public: OP-41 says it "is treated as a public descriptor" |
| Private data specifier sent | Yes                                                             | No                                                        |

Descriptor tags `0x80` to `0xFE` are user-defined in DVB
([GStreamer's MPEG-TS descriptor list](https://grok.dragonflybsd.org/xref/dports/multimedia/gstreamer1-plugins-mpeg2enc/gst-plugins-bad-1.16.2/gst-libs/gst/mpegts/gstmpegtsdescriptor.h)),
so a receiver needs the private data specifier to know whose `0x83` it is
looking at. TSDuck
documents the DTG descriptor as one that "must be preceded by the DTG/OFCOM
PDS", and a 2021 FFmpeg patch discussion lists the per-country rules: UK
`0x0000233A` + `0x83`, most of Europe `0x00000028` + `0x83`, Nordig
`0x00000029`, Australia `0x83` with no private data specifier at all
([TSDuck](https://tsduck.io/doxy/d6/dd0/tsDTGLogicalChannelDescriptor_8h.html);
[ffmpeg-devel, May 2021](https://ffmpeg.org/pipermail/ffmpeg-devel/2021-May/280305.html)).

So the likely chain is:

1. Perth broadcasters send `0x83` with no private data specifier.
2. The UK firmware only reads `0x83` after the UK specifier, so it finds
   no LCNs and stores `65535`.
3. The UK rules then apply to services without a number. Under the DTG
   specification, a receiver numbers services that have no LCN, or can't
   take the one they have, itself, in the variant or manufacturer ranges.
   Digital UK reserves **800–999 for manufacturer use**
   ([DTG wording quoted on the hummy.tv forum](https://hummy.tv/forum/threads/renumber-channel-re-numbering-extension.1821/post-23743);
   [Digital UK LCN ranges, 2019](https://www.everyonetv.co.uk/sites/everyonetv/files/2022-10/Allocation-LCN-Freeview-accessible-TV-guide-consultation-statement-2019.pdf)).
   UK viewers see the same range for duplicate regional channels
   ([ukfree.tv](https://ukfree.tv/extras/inreplyto/771001)).

This fits everything in the export, but we haven't captured the Perth NIT to
prove step 1, and Samsung's firmware isn't public, so step 2 is inferred.
TSDuck also models the Australian descriptor as one that follows a Free TV
private data specifier, which disagrees with OP-41 and the FFmpeg
discussion. Either way the UK firmware sees no UK specifier.

For comparison, OP-41 tells Australian receivers to put services without an
LCN, or that lose an LCN clash, at the next free number in **350–399**.

## Reception is fine: VHF works

Before looking at the export, we assumed the TV was UHF only, because UK
TVs were said to be. It isn't. The `CHNL` table records the frequency each
multiplex was found on, and all five Perth multiplexes are on VHF Band III:

| Multiplex | Frequency | Channel |
| --------- | --------- | ------- |
| Seven     | 177.5 MHz | VHF 6   |
| SBS       | 184.5 MHz | VHF 7   |
| Nine      | 191.5 MHz | VHF 8   |
| Ten       | 219.5 MHz | VHF 11  |
| ABC       | 226.5 MHz | VHF 12  |

The TV can only have read the service names by tuning those frequencies.

The frequencies match the Australian Band III plan: channels 6 to 12 cover
174–230 MHz in 7 MHz steps, so channel 6 is 174–181 MHz (centre 177.5) and
channel 12 is 223–230 MHz (centre 226.5)
([ACMA broadcasting bands](https://www.acma.gov.au/bands-broadcasting-services);
[Wikipedia: Band III](https://en.wikipedia.org/wiki/Band_III)).

The "UK TVs are UHF only" advice was true of **analogue** sets: UK
analogue TV used only UHF
([International TV channel tables](https://www.stjarnhimlen.se/tv/tv.html)).
UK digital terrestrial TV is also broadcast on UHF
([UK Parliament POSTnote 264, 2006](https://webtest.parliament.uk/globalassets/documents/post/postpn264.pdf)).
But a digital TV's tuner is a single chip covering both bands. Common
silicon tuners are specified for **42–870 MHz** with 6, 7 and 8 MHz channel
bandwidths, for example the
[NXP TDA18212](https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2401161510_NXP-Semicon-TDA18212HN-M-C1557_C3823883.pdf)
and [Silicon Labs Si2157](https://dtsheet.com/doc/1769253/si2157-data-short).
We don't know which tuner this TV uses.

Whether the firmware scans VHF is a separate question, and European
firmware has a reason to. Band III in Europe is eight 7 MHz channels from
174 to 230 MHz, planned for DVB-T and DAB, and Finland, Italy and Sweden use
it for DVB-T ([EBU factsheet on Band III](https://tech.ebu.ch/docs/factsheets/ebu_tech_fs_use_of_band_III.pdf)).
Germany has also used 7 MHz DVB-T on VHF
([linux-dvb, 2004](https://linuxtv.org/mailinglists/linux-dvb/2004/09-2004/msg00098.html)).
We don't know exactly which countries' plans this firmware includes, but
the export shows it scans Band III with 7 MHz channels, as Australia uses.

## Fixes considered

| Option                                               | Outcome                                                                                                                                                               |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Change the TV's country to Australia, then rescan    | The proper fix, but this model's Broadcasting menu has no Country (Area) option                                                                                       |
| Change the region in the hidden service menu         | Possible on some Samsungs, but that menu has settings that can break the TV. Kept as a last resort, not tried                                                         |
| **Edit Channels → Change Number** on the TV          | Commonly greyed out for digital channels. Samsung forum advice is to export the list and use ChanSort                                                                 |
| **ChanSort** (Windows app that edits exported lists) | Supports this file format (Samsung J/K/M/MU/Q `.zip`). Windows only, and a one-off edit: every rescan means redoing it by hand                                        |
| Favourites list                                      | Lets you reorder, but not renumber. Doesn't help someone typing numbers from a guide                                                                                  |
| Australian set-top box (around A$40–60)              | Correct numbers without any editing, and survives rescans. Worth considering, but it adds a second remote                                                             |
| **tvmap**                                            | Edits the exported list directly. The number for each channel lives in a map keyed on broadcast IDs, so after a rescan you re-apply the map instead of starting again |

## Sources

All read 31 October 2026. [sources.md](sources.md) lists everything in
one place.

- **Samsung format and fixes:**
  [ChanSort](https://github.com/PredatH0r/ChanSort) source, Samsung loader
  (`source/ChanSort.Loader.Samsung/Zip/DbSerializer.cs`, `DbChannel.cs`);
  [ChanSort Samsung wiki page](https://github.com/PredatH0r/ChanSort/wiki/Samsung)
  (J, K, M, MU and Q series export `.zip` lists);
  [SamyGO forum](https://forum.samygo.tv/viewtopic.php?p=87049)
  (Change Number greyed out; use ChanSort instead)
- **LCN signalling:** Free TV Australia
  [OP-41 Issue 9](https://www.freetv.com.au/wp-content/uploads/2024/02/Free-TV-OP-41-LCN-Descriptor-Issue-9-January-2024.pdf)
  (Jan 2024) and [Issue 8](https://www.freetv.com.au/op-41-lcn-descriptor-issue-8-july-2016)
  (Jul 2016);
  [TSDuck DTG descriptor](https://tsduck.io/doxy/d6/dd0/tsDTGLogicalChannelDescriptor_8h.html);
  [ffmpeg-devel thread](https://ffmpeg.org/pipermail/ffmpeg-devel/2021-May/280305.html)
- **UK numbering ranges:**
  [Digital UK 2019 statement](https://www.everyonetv.co.uk/sites/everyonetv/files/2022-10/Allocation-LCN-Freeview-accessible-TV-guide-consultation-statement-2019.pdf);
  [hummy.tv forum](https://hummy.tv/forum/threads/renumber-channel-re-numbering-extension.1821/post-23743);
  [ukfree.tv](https://ukfree.tv/extras/inreplyto/771001)
- **Frequencies and tuners:**
  [ACMA](https://www.acma.gov.au/bands-broadcasting-services);
  [Wikipedia: Band III](https://en.wikipedia.org/wiki/Band_III);
  [EBU Band III factsheet](https://tech.ebu.ch/docs/factsheets/ebu_tech_fs_use_of_band_III.pdf);
  [POSTnote 264](https://webtest.parliament.uk/globalassets/documents/post/postpn264.pdf);
  [stjarnhimlen.se TV tables](https://www.stjarnhimlen.se/tv/tv.html);
  NXP TDA18212 and Silicon Labs Si2157 datasheets (links above)
