# Samsung channel-list export format

What's inside `Channel_list_T-KTMDEUC-1410.1.zip`, exported from a
UE40MU6400. The layout matches what ChanSort's Samsung `.zip` loader
expects. Other firmware versions may differ; see [Caveats](#caveats).

## The zip

| Entry (in order) | Size | What it is |
|---|---:|---|
| `dvbt` | 27 KB | SQLite: antenna (DVB-T) channels. The only file tvmap changes |
| `sat` | 68 KB | SQLite: satellite and transponder tables (`SAT`, `SAT_TP`, `SAT_HT`, `SAT_HS`). Present although satellite isn't used |
| `ipsrv` | 1.9 MB | SQLite: internet channels (`CHNL`, `SRV`, `SRV_IP`, `SRV_FAV`), probably Samsung TV Plus. Not looked at further |
| `metadata.xml` | 187 B | `ANALOG_COUNTRY` 7, `DIGITAL_COUNTRY` 4670034 (`0x474252` = `GBR`), `SEI_VERSION` 64 |

- Every entry is deflated, with a zero DOS timestamp (shows as
  `1980-00-00`) and "made by" version 0.0.
- The databases have **no file extension**. tvmap treats any top-level,
  extension-less entry that contains `SRV`, `SRV_DVB` and `CHNL` tables as
  a broadcast database. A cable-tuned TV would presumably have a different
  entry name; we've only seen `dvbt`.
- When tvmap writes a new zip, it raw-copies every entry except the
  broadcast database, so their compressed bytes are identical, and it keeps
  the entry order. The TV seems to want the original file name on the
  stick, so the output keeps that name in a separate folder.

## The `dvbt` database

- **Text encoding:** UTF-16LE, page size 512.
- **Version info:** `user_version` 4194755, last written by SQLite 3.8.10.
  After tvmap's write, the header records the bundled SQLite version
  instead. The TV accepted that without complaint.
- **Tables:** `CHNL`, `SRV`, `SRV_ANL`, `SRV_DVB`, `SRV_DVB_EXT`,
  `SRV_FAV`, `PROV`, `DEF_AUTH`, `SGNL`, `sqlite_sequence`.

### Tables that matter

**`CHNL`**: one row per multiplex (5 in Perth).

| Column | Meaning | Perth values |
|---|---|---|
| `chId` | Key, referenced by `SRV.chId` | 4357506 … 4420226 |
| `freq` | Centre frequency in kHz | 177500 … 226500 |
| `chType` | Signal type | 2 |
| `band`, `mod`, `dig` | Band, modulation, digital flag | 2, 10, 1 |
| `chNum` | Unknown; differs per multiplex | 2550, 2690, 2830, 3390, 3530 |

**`SRV`**: one row per service (58 in Perth).

| Column | Meaning |
|---|---|
| `srvId` | Key. Large 64-bit values (around 2.4 × 10¹⁸); store as `i64` |
| `chId` | The multiplex it's on (`CHNL.chId`) |
| `major` | **The channel number the TV shows.** This is what tvmap changes |
| `minor` | 65534 everywhere |
| `progNum` | DVB service ID |
| `tsid` | Transport stream ID. In this schema it's in `SRV`, not `SRV_DVB` |
| `srvName` | Name, in a byte-swapped encoding: see below |
| `srvType` | DVB service type (see below) |
| `hidden`, `hideGuide` | Hidden from the channel list / guide. ChanSort always sets both together |
| `numSel` | Selectable with CH+/CH−. 0 means "skip" |
| `elim` | Deleted. ChanSort ignores rows where it's set; so does tvmap |
| `lockMode`, `modifiedByUser`, … | Not used by tvmap |

**`SRV_DVB`**: one row per service, joined on `srvId`.

| Column | Meaning | Perth values |
|---|---|---|
| `onid` | Original network ID | 4114 (Nine), 4115 (Seven), 4116 (Ten), 12801 (ABC), 12802 (SBS) |
| `nid` | Network ID | 12859, 12860, 12861, 12857, 12802 |
| `lcn`, `hdLcn` | Broadcaster's channel number | **65535 for every service**: the UK firmware didn't keep them |
| `shrtSrvName` | Short name | NULL |

**`SRV_FAV`**: favourites lists, as `(srvId, fav, pos)`. This TV has one
list (`fav` = 1) with five entries: ABC HD, 7, 9, 10 and SBS HD. tvmap
leaves favourites alone.

**`PROV`**: provider names: `ABC`, `SBS`, `Seven Network`, `TEN Perth`,
`Nine Network Australia`.

### Identifying a service

`(SRV_DVB.onid, SRV.tsid, SRV.progNum)` identifies a broadcast service
uniquely, and comes from the broadcast itself, so it should survive a
rescan. `srvId` is the TV's own row key. We haven't tested whether a rescan
changes it, so tvmap only uses it within a single file, never in a map.

Names are **not** unique. Perth has three services called `7 Perth`, and
two each of `9HD Perth`, `9GemHD Perth`, `9Go!HD Perth`, `10 HD` and
`ABC TV`.

### Service types seen

| `srvType` | DVB meaning | Count |
|---:|---|---:|
| 1 | Digital TV, MPEG-2 SD | 8 |
| 22 (0x16) | H.264 SD TV | 8 |
| 25 (0x19) | H.264 HD TV | 25 |
| 2 | Digital radio (MPEG-1 Layer II) | 16 |
| 10 (0x0A) | Advanced codec radio | 1 |

## Names: read them, never write them

`srvName` is a TEXT column, but the bytes inside are UTF-16 with the byte
order swapped. Read normally, `7 Perth` comes out as nonsense CJK
characters. To read a name reliably:

```sql
select cast(srvName as blob) from SRV
```

then decode the bytes as UTF-16BE and drop trailing NULs.

**Writing names is where files get corrupted.** Write the bytes as a BLOB
and the column's type changes, which the TV rejects. Write them as TEXT and
SQLite re-encodes them. ChanSort's source has a long comment on the
workarounds and how they broke between library versions, and one firmware
format ("1242") can't be written correctly at all. tvmap never writes
names or IDs.

## What tvmap writes

Inside one transaction, and only for services where something changed:

| Setting | SQL |
|---|---|
| Number | `update SRV set major = ? where srvId = ?` |
| Hidden | `update SRV set hidden = ?, hideGuide = ? where srvId = ?` |
| Skip | `update SRV set numSel = ? where srvId = ?` |

followed by `REINDEX`. ChanSort does this too, because some Samsung files
ship with stale indexes.

The tests compare every column of every table before and after. Applying
the Perth map changes `SRV.major` and nothing else. See
[verification.md](verification.md).

## Caveats

- Everything here is from one TV and one firmware (T-KTMDEUC-1410.1). A
  different firmware may use a different schema; ChanSort's loader handles
  several variants, tvmap only this one.
- On some setups, Samsung hides **Transfer Channel List** from the menu.
  One user on Samsung's forum reported it appeared after resetting the TV
  and running a full (Complete List) scan. On this TV it was there from the
  start.

## Sources

- ChanSort, `source/ChanSort.Loader.Samsung/Zip/DbSerializer.cs` (queries,
  update statements, `REINDEX`, the name-encoding notes) and `DbChannel.cs`
  (column meanings): <https://github.com/PredatH0r/ChanSort>
- Our own inspection of the export (`tests/fixtures/`)
