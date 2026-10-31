# Research notes

What we found while building tvmap. The project README covers how to use
the tool; these files cover why it works the way it does.

| File                                                             | What's in it                                                                                                         |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| [background.md](background.md)                                   | The problem, why a UK Samsung puts Australian channels at 800+, and the fixes we looked at before writing a tool     |
| [samsung-channel-list-format.md](samsung-channel-list-format.md) | The `Channel_list_*.zip` export: files, SQLite schema, the name-encoding trap, and exactly what tvmap writes         |
| [perth-channels.md](perth-channels.md)                           | Every service in the Perth export, with IDs, frequencies and the number tvmap gives it, plus the numbering decisions |
| [tv-procedures.md](tv-procedures.md)                             | Exporting and importing on the TV, settings to change afterwards, and the channel-entry delay                        |
| [verification.md](verification.md)                               | How the write path was checked before anything went near the TV                                                      |

Everything here comes from one TV: a UK UE40MU6400 on firmware
T-KTMDEUC-1410.1, tuned in Perth in October 2026. Where a statement is a
guess rather than something we saw, it says so.
