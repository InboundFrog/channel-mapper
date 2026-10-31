//! Matching services against a map and working out what to change.

use crate::db::{Change, Service, ServiceKey, Settings};
use crate::mapfile::{MapEntry, MapFile};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// IDs match and the name looks right.
    Matched,
    /// IDs match but the name is quite different from the map's.
    NameChanged { expected: String },
    /// IDs not in the map, but exactly one map entry with this name whose IDs
    /// are no longer broadcast. Probably the broadcaster changed a service ID.
    MatchedByName { was: ServiceKey },
    /// Not in the map; number taken from the broadcast LCN.
    FromLcn,
    /// Not in the map. Left as-is until someone picks a number.
    Unmatched,
    /// Edited by hand in the TUI.
    Manual,
}

impl Status {
    pub fn needs_attention(&self) -> bool {
        matches!(
            self,
            Status::NameChanged { .. } | Status::MatchedByName { .. } | Status::Unmatched
        )
    }

    pub fn describe(&self) -> String {
        match self {
            Status::Matched => "ok".into(),
            Status::NameChanged { expected } => format!("name changed (map: '{expected}')"),
            Status::MatchedByName { was } => format!("matched by name (IDs were {was})"),
            Status::FromLcn => "from broadcast LCN".into(),
            Status::Unmatched => "not in map".into(),
            Status::Manual => "edited".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub svc: Service,
    pub status: Status,
    /// `None` keeps the number the TV assigned.
    pub number: Option<i64>,
    pub hidden: bool,
    pub skip: bool,
}

impl Row {
    pub fn target(&self) -> Settings {
        Settings {
            number: self.number.unwrap_or(self.svc.major),
            hidden: self.hidden,
            skip: self.skip,
        }
    }

    pub fn is_changed(&self) -> bool {
        self.target() != self.svc.current()
    }
}

/// Lower-case, alphanumerics only, and without a trailing city name so
/// "7two Perth" and "7two" compare equal.
pub fn normalise(name: &str) -> String {
    let s: String = name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    s.strip_suffix("perth").map(str::to_string).unwrap_or(s)
}

pub fn names_similar(a: &str, b: &str) -> bool {
    let (a, b) = (normalise(a), normalise(b));
    if a.is_empty() || b.is_empty() {
        return a == b;
    }
    a == b || a.contains(&b) || b.contains(&a) || strsim::normalized_levenshtein(&a, &b) >= 0.6
}

pub struct Plan {
    pub rows: Vec<Row>,
}

impl Plan {
    pub fn build(services: Vec<Service>, map: &MapFile) -> Self {
        let by_key: HashMap<ServiceKey, &MapEntry> =
            map.channels.iter().map(|e| (e.key(), e)).collect();
        let present: HashSet<ServiceKey> = services.iter().map(|s| s.key).collect();

        // Map entries whose service no longer exists, indexed by normalised
        // name, for the "service ID changed" fallback.
        let mut orphans: HashMap<String, Vec<&MapEntry>> = HashMap::new();
        for e in &map.channels {
            if !present.contains(&e.key()) {
                orphans.entry(normalise(&e.name)).or_default().push(e);
            }
        }

        let rows = services
            .into_iter()
            .map(|svc| {
                let from = |e: &MapEntry, status: Status| Row {
                    number: e.number,
                    hidden: e.hidden,
                    skip: e.skip,
                    status,
                    svc: svc.clone(),
                };
                if let Some(e) = by_key.get(&svc.key) {
                    let status = if names_similar(&svc.name, &e.name) {
                        Status::Matched
                    } else {
                        Status::NameChanged {
                            expected: e.name.clone(),
                        }
                    };
                    return from(e, status);
                }
                if let Some([e]) = orphans.get(&normalise(&svc.name)).map(Vec::as_slice) {
                    return from(e, Status::MatchedByName { was: e.key() });
                }
                let lcn = (1..65535).contains(&svc.lcn).then_some(svc.lcn);
                Row {
                    status: if lcn.is_some() {
                        Status::FromLcn
                    } else {
                        Status::Unmatched
                    },
                    number: lcn,
                    hidden: svc.hidden,
                    skip: svc.skip,
                    svc,
                }
            })
            .collect::<Vec<_>>();

        // Two services can't both claim the same orphaned map entry.
        let mut rows = rows;
        let mut claims: HashMap<ServiceKey, usize> = HashMap::new();
        for r in &rows {
            if let Status::MatchedByName { was } = r.status {
                *claims.entry(was).or_default() += 1;
            }
        }
        for r in &mut rows {
            if let Status::MatchedByName { was } = r.status {
                if claims[&was] > 1 {
                    r.status = Status::Unmatched;
                    r.number = None;
                    r.hidden = r.svc.hidden;
                    r.skip = r.svc.skip;
                }
            }
        }

        let mut plan = Plan { rows };
        plan.sort();
        plan
    }

    /// Sort by the number each service will end up with.
    pub fn sort(&mut self) {
        self.rows.sort_by_key(|r| (r.target().number, r.svc.key));
    }

    /// Numbers used by more than one service, with the row indices using them.
    pub fn conflicts(&self) -> BTreeMap<i64, Vec<usize>> {
        let mut by_num: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
        for (i, r) in self.rows.iter().enumerate() {
            by_num.entry(r.target().number).or_default().push(i);
        }
        by_num.retain(|_, v| v.len() > 1);
        by_num
    }

    /// Changes grouped by database entry name.
    pub fn changes(&self) -> BTreeMap<String, Vec<Change>> {
        let mut out: BTreeMap<String, Vec<Change>> = BTreeMap::new();
        for r in self.rows.iter().filter(|r| r.is_changed()) {
            out.entry(r.svc.db.clone()).or_default().push(Change {
                srv_id: r.svc.srv_id,
                old: r.svc.current(),
                new: r.target(),
            });
        }
        out
    }

    /// A map that reproduces this plan after a rescan. Services left on the
    /// TV's own number are saved without a number, so the TV's next choice
    /// stands.
    pub fn to_map(&self, region: Option<String>) -> MapFile {
        let mut channels: Vec<MapEntry> = self
            .rows
            .iter()
            .map(|r| MapEntry {
                onid: r.svc.key.onid,
                tsid: r.svc.key.tsid,
                sid: r.svc.key.sid,
                name: r.svc.name.clone(),
                number: r.number,
                hidden: r.hidden,
                skip: r.skip,
            })
            .collect();
        channels.sort_by_key(|e| (e.number.unwrap_or(i64::MAX), e.key()));
        MapFile { region, channels }
    }
}
