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
        let title = format!(
            " tvmap  {}  |  {} services, {changed} changed, {attention} to check, {} conflicting{}{}",
            self.s.zip.file_name().unwrap_or_default().to_string_lossy(),
            plan.rows.len(),
            conflicts.len(),
            if self.only_attention { "  [problems only]" } else { "" },
            if self.dirty { "  *unsaved*" } else { "" },
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
