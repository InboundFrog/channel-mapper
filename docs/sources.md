# Sources

Everything the research notes rely on, grouped by topic. All links were read
on 31 October 2026. "Used for" says which claim each one backs.

## The TV's own data

| Source                                                         | Used for                                                                                                            |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| The export, `tests/fixtures/Channel_list_T-KTMDEUC-1410.1.zip` | Schema, IDs, names, service types, frequencies, `lcn` = 65535, `DIGITAL_COUNTRY` = GBR, the alphabetical 800+ order |
| What we saw on the UE40MU6400 (firmware T-KTMDEUC-1410.1)      | Menu paths, no Country (Area) setting, reboot on import, the channel-entry delay                                    |

## Samsung channel lists

| Source                                                                                                                                                        | Used for                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| [ChanSort](https://github.com/PredatH0r/ChanSort) source: `source/ChanSort.Loader.Samsung/Zip/DbSerializer.cs`, `DbChannel.cs`                                | Which entries in the zip are databases, the queries and update statements, `REINDEX`, column meanings, the name-encoding trap |
| [ChanSort README](https://github.com/PredatH0r/ChanSort)                                                                                                      | USB stick: FAT32, 32 GB or less                                                                                               |
| [ChanSort wiki: Samsung](https://github.com/PredatH0r/ChanSort/wiki/Samsung)                                                                                  | J/K/M/MU/Q series export `.zip`; Transfer Channel List; PIN `0000`                                                            |
| [SamyGO forum: renumber digital channels](https://forum.samygo.tv/viewtopic.php?p=87049)                                                                      | Change Number greyed out for digital channels; advice to use ChanSort                                                         |
| [Samsung Community: Channel Editing / Transfer Channel List](https://eu.community.samsung.com/t5/tv/channel-editing-transfer-channel-list/td-p/364538/page/2) | Transfer Channel List missing on some setups, appearing after a reset and full scan                                           |
| [Samsung TV e-Manual: Transfer Channel List](https://www.manualslib.com/manual/816924/Samsung-Tv.html?page=23)                                                | PIN prompt; Import from / Export to USB                                                                                       |
| [Samsung QN92A e-Manual: changing channels](https://www.manualslib.com/manual/3392312/Samsung-Qn92a.html?page=239)                                            | Enter the number and select, or wait                                                                                          |
| [Sony: faster channel switching on BRAVIA 3](https://www.sony.ie/electronics/support/televisions-projectors/articles/00358103)                                | Other brands also wait for more digits; pressing Enter changes at once                                                        |

## Channel numbers (LCNs)

| Source                                                                                                                                                                                                                                | Used for                                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Free TV Australia, [OP-41 Issue 9](https://www.freetv.com.au/wp-content/uploads/2024/02/Free-TV-OP-41-LCN-Descriptor-Issue-9-January-2024.pdf) (January 2024)                                                                         | Descriptor tag `0x83`, treated as public; network number ranges (Table 3), Ten on 1; receivers put unnumbered services at 350–399 |
| Free TV Australia, [OP-41 Issue 8](https://www.freetv.com.au/op-41-lcn-descriptor-issue-8-july-2016) (July 2016)                                                                                                                      | The "DVB triple" (onid, tsid, sid); the descriptor comes from the UK's DTG                                                        |
| Free TV Australia, [OP-40 Issue 4](https://www.freetv.com.au/wp-content/uploads/2019/08/OP-40-Allocation-of-DVB-Service-Information-Codes-Issue-4-Sept-2017-v2..pdf) (September 2017)                                                 | How network IDs and other DVB codes are allocated in Australia                                                                    |
| Free TV Australia, [DTTB Service Information Register introduction](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-Introduction.pdf) (September 2021) | The DTG granted permission to use its descriptor in Australia                                                                     |
| [TSDuck: DTG logical_channel_descriptor](https://tsduck.io/doxy/d6/dd0/tsDTGLogicalChannelDescriptor_8h.html)                                                                                                                         | The UK descriptor is private and needs the DTG/Ofcom private data specifier                                                       |
| [TSDuck: Australia logical_channel_descriptor](https://tsduck.io/doxy/df/d0c/tsAustraliaLogicalChannelDescriptor_8h.html)                                                                                                             | TSDuck's model of the Australian descriptor (disagrees with OP-41 on the private data specifier)                                  |
| [ffmpeg-devel, May 2021](https://ffmpeg.org/pipermail/ffmpeg-devel/2021-May/280305.html)                                                                                                                                              | Private data specifier per country: UK `0x0000233A`, Europe `0x00000028`, Nordig `0x00000029`, Australia none                     |
| [GStreamer MPEG-TS descriptor list](https://grok.dragonflybsd.org/xref/dports/multimedia/gstreamer1-plugins-mpeg2enc/gst-plugins-bad-1.16.2/gst-libs/gst/mpegts/gstmpegtsdescriptor.h)                                                | Tags `0x80`–`0xFE` are user-defined; `0x83` is the DTG logical channel descriptor                                                 |
| [Digital UK LCN statement, 2019](https://www.everyonetv.co.uk/sites/everyonetv/files/2022-10/Allocation-LCN-Freeview-accessible-TV-guide-consultation-statement-2019.pdf)                                                             | UK range 800–999 reserved for manufacturer use                                                                                    |
| [hummy.tv forum](https://hummy.tv/forum/threads/renumber-channel-re-numbering-extension.1821/post-23743)                                                                                                                              | Quotes the DTG rule that receivers number services without a usable LCN in the variant or manufacturer ranges                     |
| [ukfree.tv](https://ukfree.tv/extras/inreplyto/771001)                                                                                                                                                                                | UK receivers put duplicate regional channels in 800–899                                                                           |
| Wikipedia, [List of digital television channels in Australia](https://en.wikipedia.org/wiki/List_of_digital_television_channels_in_Australia)                                                                                         | Service-by-service numbers for Perth                                                                                              |

## Frequencies and tuners

| Source                                                                                                                              | Used for                                                                                 |
| ----------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| [ACMA: bands for broadcasting services](https://www.acma.gov.au/bands-broadcasting-services)                                        | Australian VHF TV channels 6–12 are 174–230 MHz                                          |
| Wikipedia, [Band III](https://en.wikipedia.org/wiki/Band_III)                                                                       | Australian Band III channel plan, 7 MHz channels; European Band III                      |
| [EBU factsheet: use of Band III](https://tech.ebu.ch/docs/factsheets/ebu_tech_fs_use_of_band_III.pdf)                               | Band III in Europe planned for DVB-T and DAB; Finland, Italy and Sweden use it for DVB-T |
| [linux-dvb mailing list, 2004](https://linuxtv.org/mailinglists/linux-dvb/2004/09-2004/msg00098.html)                               | Germany broadcast DVB-T on VHF with 7 MHz channels                                       |
| [International TV channel tables (stjarnhimlen.se)](https://www.stjarnhimlen.se/tv/tv.html)                                         | UK analogue TV used only UHF                                                             |
| [UK Parliament POSTnote 264 (2006)](https://webtest.parliament.uk/globalassets/documents/post/postpn264.pdf)                        | UK digital terrestrial TV is transmitted in UHF                                          |
| [NXP TDA18212 datasheet](https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2401161510_NXP-Semicon-TDA18212HN-M-C1557_C3823883.pdf) | Typical silicon tuner: 42–870 MHz, 6/7/8 MHz bandwidths                                  |
| [Silicon Labs Si2157 data short](https://dtsheet.com/doc/1769253/si2157-data-short)                                                 | Another tuner with the same 42–870 MHz range                                             |

## DVB service types

| Source                                                                                                                                        | Used for                                                                 |
| --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| [ETSI EN 300 468 V1.8.1 contents](https://iteh.es/catalog/standards/etsi/27066f04-432d-4487-a84a-0b5d93237196/etsi-en-300-468-v1-8-1-2008-07) | `service_type` is defined in EN 300 468, with an annex on assigning it   |
| [mythtv-dev, December 2005](https://lists.mythtv.org/pipermail/mythtv-dev/2005-December/042205.html)                                          | Meanings of `0x11`, `0x16`, `0x19`, `0x0A`; "advanced codec" means H.264 |
| [MythTV service type constants](https://code.mythtv.org/doxygen/classServiceDescriptorMapping.html)                                           | The same values as code                                                  |

## Other cities

| Source                                                                                                                                                                                                                                                            | Used for                                                                                                                   |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Free TV Australia, DTTB Service Information Register [Issue 16](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-NSW-ACT-.pdf) (June 2021), state sheets NSW-ACT, QLD, Vic-SA, NT-WA-Tas | Service IDs and LCNs for the untested city maps; Perth column checked against the export (5/5 multiplexes, 46/58 services) |
| Free TV Australia, DTTB Service Information Register [Issue 16 introduction](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-Introduction-.pdf)                                         | The register is meant for broadcasters, and its values may change                                                          |
| Free TV Australia, DTTB Service Information Register [Issue 18](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-NSW-ACT.pdf) (September 2021, draft), same four sheets             | The newest issue we found; differs from Issue 16 in three places, none changing a guide number                             |
| David Zou, [Sydney Digital TV (DVB-T)](https://davidzou.com/articles/sydney-digital-tv-dvb-t) (December 2015)                                                                                                                                                     | Sydney's five multiplexes (ONID/TSID) and four SBS service IDs match the register                                          |

All sheet links are in [city-maps.md](city-maps.md).

## Not verified

- **Which tuner chip the UE40MU6400 uses.** The datasheets show what's
  typical, not what's in this TV.
- **Exactly how Samsung's UK firmware handles `0x83`.** The chain in
  [background.md](background.md) is inferred from the standards and the
  export. We haven't captured the Perth Network Information Table or seen
  the firmware.
- **SBS HD's mainland number.** Wikipedia doesn't state it; we inferred it
  from the Tasmanian note.
- **The city maps.** None has been tried on a TV, and the register is from
  2021. See [city-maps.md](city-maps.md#not-verified).
