# Other cities

None of the maps under `maps/au/` apart from Perth have been tested on a
TV. This covers where their IDs come from, how far that data has been
checked, and how their numbers were chosen.

## The register

Free TV Australia publishes a **DTTB Service Information Register**: Excel
sheets saved as PDF, one per group of states, with every value in hex.
Each sheet has a column per market. A multiplex row gives its ONID, NID
and TSID, and each service row under it gives the service's SID and LCN.

| Issue               | Date           | Files                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 16                  | June 2021      | [Introduction](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-Introduction-.pdf), [NSW-ACT](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-NSW-ACT-.pdf), [QLD](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-QLD.pdf), [Vic-SA](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-Vic-SA.pdf), [NT-WA-Tas](https://www.freetv.com.au/wp-content/uploads/2021/06/Free-TV-Australia-DTTB-Service-Information-Register-Ver-16-June-2021-NT-WA-Tas-.pdf)                       |
| 18 (marked "Draft") | September 2021 | [Introduction](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-Introduction.pdf), [NSW-ACT](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-NSW-ACT.pdf), [QLD](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-QLD.pdf), [Vic-SA](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-Vic-SA.pdf), [NT-WA-Tas](https://www.freetv.com.au/wp-content/uploads/2021/09/Free-TV-Australia-DTTB-Service-Information-Register-Ver-18-September-2021-NT-WA-Tas.pdf) |

Free TV's
[Technology & Innovation](https://www.freetv.com.au/what-we-do/technology-innovation/)
page didn't link these files in its HTML when we checked. We found them
through search and by trying the file-name pattern. Issue 18 is the newest
we found. The file names aren't consistent (some have a trailing `-`), so
don't expect a newer issue to follow the same pattern.

The introduction sheet sets expectations. It says the register "was not
intended to be a resource for any parties other than television
broadcasters", and that its values "may be subject to revision or change
of status from time to time". We only take the ID numbers from it, credit
it, and mark every map built from it as untested.

## How well it matches reality

### Perth, against the real export

Both issues give the same Perth column, and it lines up with the export
in `tests/fixtures/`:

- **All 5 Perth multiplexes match** (ONID and TSID): ABC 12801/609, SBS
  12802/800, Seven 4115/1286, Nine 4114/1120, Ten 4116/1543.
- **46 of the 58 services match** exactly, on onid/tsid/sid.
- **12 services are in the export but not the register:**
  - `ABCTV HD` (741)
  - ABC radio: ABC KIDS Listen (744), triple j (745), triple j Unearthed
    (746), ABC Classic (747), ABC Country (748), ABC PERTH (749), ABC RN
    (750), ABC NewsRadio (751)
  - `NITV HD` (809)
  - `TVSN` on Seven (1385)
  - `10 HD +1` (1666)
- **6 services are in the register but not the export:**
  - `7mate` SD (1379, LCN 73)
  - SBS Arabic24 (813, LCN 36)
  - SBS Radio 1 and 2 (814, 815, LCNs 37 and 38)
  - SBS Chill (816, LCN 39)
  - WTV (3585, LCN 44), Perth's community station, on its own multiplex
    (12862/3584)

The register's channel names and LCNs are out of date (it still lists
`ABC1`, `9SD` and `SBS VICELAND HD`), but the IDs are not.

### Sydney, against a 2015 capture

[David Zou's *Sydney Digital TV (DVB-T)*](https://davidzou.com/articles/sydney-digital-tv-dvb-t)
(26 December 2015) has a transmitter table and per-multiplex dumps. Its
"NID" column holds what the register calls the ONID. All five Sydney
multiplexes agree with the register's Sydney column:

| Network | ONID (page / register) | TSID (page / register) |
| ------- | ---------------------- | ---------------------- |
| ABC     | 4112 / `0x1010`        | 545 / `0x0221`         |
| SBS     | 12802 / `0x3202`       | 768 / `0x0300`         |
| Seven   | 4115 / `0x1013`        | 1282 / `0x0502`        |
| Nine    | 4114 / `0x1012`        | 1056 / `0x0420`        |
| Ten     | 4116 / `0x1014`        | 1538 / `0x0602`        |

SBS service IDs on the page also match the register: SBS ONE `0x301`,
Food `0x304`, NITV `0x305`, SBS HD `0x306`. So the Sydney IDs held from
2015 to 2021. We haven't checked them against anything newer.

We also found some igorfuna.com pages titled as Australian multiplex
analyses. When fetched, they only returned the site's landing page, so
nothing here relies on them.

## Issue 16 versus Issue 18

The maps were generated from Issue 16. Issue 18 differs in three places,
none of which change a guide number:

| Map       | Difference                                                                       |
| --------- | -------------------------------------------------------------------------------- |
| Canberra  | The service on LCN 56 (sid 2183) is `Sky News Regional`, not `Aspire`. Same IDs. |
| Melbourne | Nine's two 9Go! simulcasts (sid 1074 and 1079) swap LCNs 93 and 99               |
| Adelaide  | The same swap for 9Go! (sid 1107 and 1112)                                       |

The 9Go! swaps only change which copy of the same programme lands on 93.

`tools/freetv_maps.py` reads both issues. Issue 18 is laid out
differently: it's at half the scale, the ONID/NID/TSID/SID headings are on
one row, and the NT-WA-Tas sheet has no "Service" heading on the
"Broadcaster" row. The script measures distances in multiples of the
sheet's text height, so it copes with both. Run on Issue 16 it reproduces
the committed maps exactly; run on Issue 18 it gives the three differences
above.

## How the numbers are chosen

`tools/freetv_maps.py` applies these rules. The README says how to run it.

1. **Metro networks (ABC, SBS, Seven, Nine, Ten).** Each network runs the
   same line-up in every metro market, so a service's register LCN
   identifies it across cities. Each register service takes the name and
   number of the same service in the tested Perth map, with "Perth"
   replaced by the city's name. For example, the Sydney service on
   register LCN 90 (`9HD`) gets Perth's 9.
2. **Services Perth no longer has are left out.** That covers SBS Arabic24
   and Radio 1/2 (36–38), SBS Chill (39) and `7mate` SD (73). They're
   assumed to be gone everywhere. If one is still broadcast, it shows as
   "not in map".
3. **Regional networks** keep the register's LCN. Where the register has
   an SD and an HD copy of the main channel, they're swapped so HD gets
   the main number, as in Perth:

   | City     | Network                 | Swap  |
   | -------- | ----------------------- | ----- |
   | Canberra | Southern Cross 10       | 5, 50 |
   | Canberra | Prime7                  | 6, 60 |
   | Canberra | WIN                     | 8, 80 |
   | Hobart   | Southern Cross (Seven)  | 6, 60 |
   | Hobart   | Southern Cross 10 (TDT) | 5, 50 |
   | Hobart   | WIN                     | 8, 80 |
   | Darwin   | Southern Cross (Seven)  | 7, 70 |

   Darwin's Ten (DDT) already has the HD copy on 10, so it isn't swapped.
4. **ABC HD and NITV HD aren't in the register.** As in Perth, ABC's SD
   copy is on 20 and NITV's on 36, which leaves 2 and 34 free. Until
   someone sets the HD copies in the editor, nothing is on 2 or 34. Each
   map's header says so.
5. **Every map must be valid.** The script stops if a map would use a
   number twice, and `tests/unit.rs` loads every map under `maps/`.

## Not verified

- **No city map has been tried on a TV.** Expect yellow rows in the editor.
- **Anything since 2021.** Services that started after the register (like
  the 12 in Perth), changed IDs, and closed services all show up only
  in the editor.
- **Regional affiliations.** Canberra, Hobart and Darwin's commercial
  networks are taken as the register has them in 2021. We haven't checked
  whether ownership or affiliation changes since then moved their numbers.
- **Names.** Apart from the metro names copied from Perth, names are the
  register's labels with the city filled in. A different name on the TV
  only raises a "name changed" flag; the number still applies.
- **The rule-1 assumption.** That a register LCN identifies the same
  service in every metro market is based on the register's own row labels,
  which are the same for every market, not on exports from other cities.

## Sources

All read 31 October 2026. See also [sources.md](sources.md).

- Free TV Australia, DTTB Service Information Register, Issues 16 and 18
  (links in the table above)
- David Zou, [Sydney Digital TV (DVB-T)](https://davidzou.com/articles/sydney-digital-tv-dvb-t),
  26 December 2015
- The export in `tests/fixtures/` (the Perth comparison)
