//! Interactive editor.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row as TRow, Table, TableState};
use ratatui::{DefaultTerminal, Frame};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tvmap::plan::{Plan, Row, Status};

pub struct Session {
    pub zip: PathBuf,
    pub out: PathBuf,
    pub map_out: PathBuf,
    pub region: Option<String>,
    pub plan: Plan,
}

enum Mode {
    Normal,
    /// Typing a channel number for the selected row.
    Number(String),
    ConfirmQuit,
}

struct App {
    s: Session,
    /// What the map proposed for each service, for `r` (revert).
    proposed: HashMap<i64, (Option<i64>, bool, bool, Status)>,
    table: TableState,
    mode: Mode,
    only_attention: bool,
    dirty: bool,
    message: String,
}

impl App {
    fn new(s: Session) -> Self {
        let proposed = s
            .plan
            .rows
            .iter()
            .map(|r| (r.svc.srv_id, (r.number, r.hidden, r.skip, r.status.clone())))
            .collect();
        let mut table = TableState::default();
        table.select(Some(0));
        let attention = s
            .plan
            .rows
            .iter()
            .filter(|r| r.status.needs_attention())
            .count();
        App {
            s,
            proposed,
            table,
            mode: Mode::Normal,
            only_attention: false,
            dirty: true,
            message: format!("{attention} service(s) need a look. Press ? for keys."),
        }
    }

    fn conflict_set(&self) -> HashSet<usize> {
        self.s.plan.conflicts().into_values().flatten().collect()
    }

    /// Row indices currently shown.
    fn visible(&self) -> Vec<usize> {
        let conflicts = self.conflict_set();
        (0..self.s.plan.rows.len())
            .filter(|i| {
                !self.only_attention
                    || self.s.plan.rows[*i].status.needs_attention()
                    || conflicts.contains(i)
            })
            .collect()
    }

    fn selected_row(&self) -> Option<usize> {
        let vis = self.visible();
        self.table.selected().and_then(|i| vis.get(i).copied())
    }

    fn select_srv(&mut self, srv_id: i64) {
        let vis = self.visible();
        if let Some(pos) = vis
            .iter()
            .position(|&i| self.s.plan.rows[i].svc.srv_id == srv_id)
        {
            self.table.select(Some(pos));
        } else {
            self.clamp();
        }
    }

    fn clamp(&mut self) {
        let n = self.visible().len();
        let sel = self.table.selected().unwrap_or(0);
        self.table
            .select(if n == 0 { None } else { Some(sel.min(n - 1)) });
    }

    /// Change the selected row, then re-sort and keep it selected.
    fn edit(&mut self, f: impl FnOnce(&mut Row)) {
        let Some(i) = self.selected_row() else { return };
        let row = &mut self.s.plan.rows[i];
        let id = row.svc.srv_id;
        f(row);
        self.dirty = true;
        self.s.plan.sort();
        self.select_srv(id);
    }

    fn jump_to_next_problem(&mut self) {
        let vis = self.visible();
        let conflicts = self.conflict_set();
        let start = self.table.selected().map_or(0, |s| s + 1);
        let found = (0..vis.len())
            .map(|k| (start + k) % vis.len().max(1))
            .find(|&p| {
                let i = vis[p];
                self.s.plan.rows[i].status.needs_attention() || conflicts.contains(&i)
            });
        match found {
            Some(p) => self.table.select(Some(p)),
            None => self.message = "Nothing left that needs a look.".into(),
        }
    }

    fn write(&mut self) {
        let result = (|| -> Result<usize> {
            for p in [&self.s.out, &self.s.map_out] {
                if let Some(d) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
                    std::fs::create_dir_all(d)?;
                }
            }
            let n = tvmap::save(&self.s.zip, &self.s.plan, &self.s.out)?;
            self.s
                .plan
                .to_map(self.s.region.clone())
                .save(&self.s.map_out)?;
            Ok(n)
        })();
        self.message = match result {
            Ok(n) => {
                self.dirty = false;
                format!(
                    "Wrote {} ({n} changed) and {}",
                    self.s.out.display(),
                    self.s.map_out.display()
                )
            }
            Err(e) => format!("Not saved: {}", e.to_string().replace('\n', " ")),
        };
    }

    /// Returns false to quit.
    fn key(&mut self, k: KeyEvent) -> bool {
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
            return false;
        }
        match &mut self.mode {
            Mode::ConfirmQuit => match k.code {
                KeyCode::Char('y') => return false,
                _ => {
                    self.mode = Mode::Normal;
                    self.message.clear();
                }
            },
            Mode::Number(buf) => match k.code {
                KeyCode::Char(c) if c.is_ascii_digit() && buf.len() < 4 => buf.push(c),
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Enter => {
                    let parsed = buf.parse::<i64>().ok().filter(|n| *n >= 1);
                    self.mode = Mode::Normal;
                    match parsed {
                        Some(n) => self.edit(|r| {
                            r.number = Some(n);
                            r.status = Status::Manual;
                        }),
                        None => self.message = "Enter a number from 1 to 9999.".into(),
                    }
                }
                _ => {}
            },
            Mode::Normal => {
                self.message.clear();
                let len = self.visible().len();
                let sel = self.table.selected().unwrap_or(0);
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if self.dirty {
                            self.mode = Mode::ConfirmQuit;
                            self.message = "Unsaved changes. Quit anyway? (y/n)".into();
                        } else {
                            return false;
                        }
                    }
                    KeyCode::Down => self.table.select(Some((sel + 1).min(len.saturating_sub(1)))),
                    KeyCode::Up => self.table.select(Some(sel.saturating_sub(1))),
                    KeyCode::PageDown => {
                        self.table.select(Some((sel + 15).min(len.saturating_sub(1))))
                    }
                    KeyCode::PageUp => self.table.select(Some(sel.saturating_sub(15))),
                    KeyCode::Home => self.table.select(Some(0)),
                    KeyCode::End => self.table.select(Some(len.saturating_sub(1))),
                    KeyCode::Enter | KeyCode::Char('e') => self.mode = Mode::Number(String::new()),
                    KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                        self.mode = Mode::Number(c.to_string())
                    }
                    KeyCode::Char('k') => self.edit(|r| {
                        r.number = None;
                        r.status = Status::Manual;
                    }),
                    KeyCode::Char('h') => self.edit(|r| {
                        r.hidden = !r.hidden;
                        r.status = Status::Manual;
                    }),
                    KeyCode::Char('s') => self.edit(|r| {
                        r.skip = !r.skip;
                        r.status = Status::Manual;
                    }),
                    KeyCode::Char('r') => {
                        let proposed = self.proposed.clone();
                        self.edit(|r| {
                            let (n, h, s, st) = proposed[&r.svc.srv_id].clone();
                            (r.number, r.hidden, r.skip, r.status) = (n, h, s, st);
                        })
                    }
                    KeyCode::Char('o') => {
                        // Accept a flagged match as fine.
                        self.edit(|r| {
                            if r.status.needs_attention() && r.number.is_some() {
                                r.status = Status::Manual;
                            }
                        })
                    }
                    KeyCode::Char('n') => self.jump_to_next_problem(),
                    KeyCode::Char('a') => {
                        let id = self.selected_row().map(|i| self.s.plan.rows[i].svc.srv_id);
                        self.only_attention = !self.only_attention;
                        match id {
                            Some(id) => self.select_srv(id),
                            None => self.clamp(),
                        }
                    }
                    KeyCode::Char('w') => self.write(),
                    KeyCode::Char('?') => {
                        self.message = "digits/Enter: set number  k: keep TV's number  h: hide  s: skip  \
                                        o: ok as-is  r: revert  n: next problem  a: problems only  w: write  q: quit"
                            .into()
                    }
                    _ => {}
                }
            }
        }
        true
    }

    fn draw(&mut self, f: &mut Frame) {
        let [head, body, foot] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .areas(f.area());

        let plan = &self.s.plan;
        let conflicts = self.conflict_set();
        let changed = plan.rows.iter().filter(|r| r.is_changed()).count();
        let attention = plan
            .rows
            .iter()
            .filter(|r| r.status.needs_attention())
            .count();
        // Mode flags first, file name last: a narrow terminal cuts off the end.
        let title =
            format!(
            " tvmap{}{}  {} services, {changed} changed, {attention} to check, {} conflicting  {}",
            if self.dirty { "  *unsaved*" } else { "" },
            if self.only_attention { "  [problems only]" } else { "" },
            plan.rows.len(),
            conflicts.len(),
            self.s.zip.file_name().unwrap_or_default().to_string_lossy(),
        );
        f.render_widget(Paragraph::new(title).reversed(), head);

        let rows: Vec<TRow> = self
            .visible()
            .into_iter()
            .map(|i| {
                let r = &plan.rows[i];
                let t = r.target();
                let mut style = Style::default();
                if r.status.needs_attention() {
                    style = style.fg(Color::Yellow);
                }
                if t.hidden {
                    style = style.add_modifier(Modifier::DIM);
                }
                if conflicts.contains(&i) {
                    style = style.fg(Color::Red).add_modifier(Modifier::BOLD);
                }
                let new = if r.is_changed() && t.number != r.svc.major {
                    Span::from(t.number.to_string()).bold()
                } else {
                    Span::from(t.number.to_string())
                };
                let flags = [(t.hidden, "hidden"), (t.skip, "skip")]
                    .iter()
                    .filter(|(on, _)| *on)
                    .map(|(_, s)| *s)
                    .collect::<Vec<_>>()
                    .join(",");
                TRow::new(vec![
                    Cell::from(r.svc.major.to_string()),
                    Cell::from(new),
                    Cell::from(r.svc.kind.label()),
                    Cell::from(r.svc.name.clone()),
                    Cell::from(flags),
                    Cell::from(r.status.describe()),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(24),
                Constraint::Length(12),
                Constraint::Min(10),
            ],
        )
        .header(
            TRow::new(["Now", "New", "Type", "Name", "Flags", "Status"])
                .bold()
                .underlined(),
        )
        .row_highlight_style(Style::default().reversed())
        .block(Block::bordered());
        f.render_stateful_widget(table, body, &mut self.table);

        let detail = self
            .selected_row()
            .map(|i| {
                let r = &plan.rows[i];
                format!(
                    "{}  ids {}  db {}  lcn {}",
                    r.svc.name,
                    r.svc.key,
                    r.svc.db,
                    if r.svc.lcn == 65535 {
                        "-".into()
                    } else {
                        r.svc.lcn.to_string()
                    }
                )
            })
            .unwrap_or_default();
        let bottom = match &self.mode {
            Mode::Number(buf) => Line::from(vec![
                Span::from(" New number: ").bold(),
                Span::from(format!("{buf}_")),
                Span::from("   Enter to set, Esc to cancel").dim(),
            ]),
            _ if !self.message.is_empty() => Line::from(format!(" {}", self.message)),
            _ => Line::from(" ?: keys  digits: set number  k: keep  h: hide  s: skip  n: next problem  w: write  q: quit").dim(),
        };
        f.render_widget(
            Paragraph::new(vec![Line::from(format!(" {detail}")).dim(), bottom]),
            foot,
        );
    }
}

fn event_loop(term: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        term.draw(|f| app.draw(f))?;
        if let Event::Key(k) = event::read()? {
            if k.kind == KeyEventKind::Press && !app.key(k) {
                return Ok(());
            }
        }
    }
}

pub fn run(session: Session) -> Result<()> {
    let mut app = App::new(session);
    let mut term = ratatui::init();
    let res = event_loop(&mut term, &mut app);
    ratatui::restore();
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tvmap::mapfile::MapFile;

    const FIXTURE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/Channel_list_T-KTMDEUC-1410.1.zip"
    );

    fn app(dir: &std::path::Path) -> App {
        let zip = PathBuf::from(FIXTURE);
        let (_, services) = tvmap::load(&zip).unwrap();
        App::new(Session {
            zip,
            out: dir.join("out.zip"),
            map_out: dir.join("map.toml"),
            region: None,
            plan: Plan::build(services, &MapFile::perth()),
        })
    }

    fn press(app: &mut App, keys: &str) {
        for c in keys.chars() {
            let code = match c {
                '\n' => KeyCode::Enter,
                'v' => KeyCode::Down,
                '^' => KeyCode::Up,
                c => KeyCode::Char(c),
            };
            assert!(app.key(KeyEvent::from(code)), "quit unexpectedly on {c:?}");
        }
    }

    fn screen(app: &mut App) -> String {
        let mut term = Terminal::new(TestBackend::new(110, 30)).unwrap();
        term.draw(|f| app.draw(f)).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn selected(app: &App) -> &Row {
        &app.s.plan.rows[app.selected_row().unwrap()]
    }

    #[test]
    fn edit_number_resorts_and_keeps_selection() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(dir.path());
        press(&mut a, "vvv"); // 1, 2, 3, -> 7 Perth
        assert_eq!(selected(&a).svc.name, "7 Perth");
        press(&mut a, "42\n");
        let r = selected(&a);
        assert_eq!((r.svc.name.as_str(), r.target().number), ("7 Perth", 42));
        assert_eq!(r.status, Status::Manual);
        assert!(screen(&mut a).contains("7 Perth"));
    }

    #[test]
    fn conflicts_block_writing_until_fixed() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(dir.path());
        press(&mut a, "72\n"); // "10" -> 72, clashes with 7two
        assert_eq!(a.conflict_set().len(), 2);
        press(&mut a, "w");
        assert!(a.message.starts_with("Not saved"), "{}", a.message);
        assert!(!dir.path().join("out.zip").exists());
        assert!(screen(&mut a).contains("2 conflicting"));

        press(&mut a, "r"); // back to the map's 1
        assert!(a.conflict_set().is_empty());
        press(&mut a, "hw");
        assert!(a.message.starts_with("Wrote"), "{}", a.message);
        assert!(!a.dirty);

        let (_, services) = tvmap::load(&dir.path().join("out.zip")).unwrap();
        let ten = services.iter().find(|s| s.key.sid == 1669).unwrap();
        assert_eq!((ten.major, ten.hidden), (1, true));
        // The saved map reproduces it.
        let map = MapFile::load(&dir.path().join("map.toml")).unwrap();
        let e = map.channels.iter().find(|e| e.sid == 1669).unwrap();
        assert_eq!((e.number, e.hidden), (Some(1), true));
    }

    #[test]
    fn keep_and_skip() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(dir.path());
        press(&mut a, "k"); // "10" keeps the TV's 800
        let r = selected(&a);
        assert_eq!((r.svc.name.as_str(), r.target().number), ("10", 800));
        press(&mut a, "s");
        assert!(selected(&a).skip);
    }

    #[test]
    fn quitting_with_unsaved_changes_asks_first() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(dir.path());
        assert!(a.key(KeyEvent::from(KeyCode::Char('q'))));
        assert!(matches!(a.mode, Mode::ConfirmQuit));
        assert!(!a.key(KeyEvent::from(KeyCode::Char('y'))));
    }

    #[test]
    fn problems_filter_and_jump() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = app(dir.path());
        press(&mut a, "n");
        assert!(a.message.contains("Nothing left"));
        press(&mut a, "a");
        assert!(a.visible().is_empty());
        assert!(screen(&mut a).contains("[problems only]"));
    }
}
