use crate::{RadioBrowserApp, SearchOptions};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use radiobrowser::ApiStation;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

// ── Warna tema ──────────────────────────────────────────────────────────────
const ACCENT: Color = Color::Rgb(220, 80, 50);
const BG: Color = Color::Rgb(18, 18, 22);
const BG2: Color = Color::Rgb(28, 28, 34);
const BG3: Color = Color::Rgb(38, 38, 46);
const FG: Color = Color::Rgb(220, 218, 210);
const FG_DIM: Color = Color::Rgb(130, 128, 122);
const GREEN: Color = Color::Rgb(80, 200, 120);
const AMBER: Color = Color::Rgb(230, 150, 40);
const BORDER: Color = Color::Rgb(60, 58, 72);
const BORDER_FOCUS: Color = Color::Rgb(220, 80, 50);

// ── State panel fokus ────────────────────────────────────────────────────────
#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Search,
    Results,
}

// ── Mode popup ───────────────────────────────────────────────────────────────
#[derive(PartialEq)]
enum Modal {
    None,
    Help,
    Error(String),
    Playing(String),
}

// ── State input search ───────────────────────────────────────────────────────
#[derive(Default)]
struct SearchForm {
    name: String,
    country: String,
    language: String,
    tag: String,
    limit: String,
    active_field: usize, // 0=name,1=country,2=language,3=tag,4=limit
    cursor_pos: usize,
}

impl SearchForm {
    fn new() -> Self {
        Self {
            limit: "20".to_string(),
            ..Default::default()
        }
    }

    fn active_value(&self) -> &str {
        match self.active_field {
            0 => &self.name,
            1 => &self.country,
            2 => &self.language,
            3 => &self.tag,
            4 => &self.limit,
            _ => "",
        }
    }

    fn active_value_mut(&mut self) -> &mut String {
        match self.active_field {
            0 => &mut self.name,
            1 => &mut self.country,
            2 => &mut self.language,
            3 => &mut self.tag,
            4 => &mut self.limit,
            _ => unreachable!(),
        }
    }

    fn push_char(&mut self, c: char) {
        let pos = self.cursor_pos;
        let val = self.active_value_mut();
        let pos = pos.min(val.len());
        val.insert(pos, c);
        self.cursor_pos = pos + 1;
    }

    fn pop_char(&mut self) {
        let pos = self.cursor_pos;
        if pos > 0 {
            let new_pos = pos - 1;
            self.active_value_mut().remove(new_pos);
            self.cursor_pos = new_pos;
        }
    }

    fn delete_char(&mut self) {
        let pos = self.cursor_pos;
        let val = self.active_value_mut();
        if pos < val.len() {
            val.remove(pos);
        }
    }

    fn sync_cursor(&mut self) {
        let len = self.active_value().len();
        self.cursor_pos = self.cursor_pos.min(len);
    }

    fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 5;
        self.sync_cursor();
    }

    fn prev_field(&mut self) {
        self.active_field = if self.active_field == 0 {
            4
        } else {
            self.active_field - 1
        };
        self.sync_cursor();
    }

    fn has_filter(&self) -> bool {
        !self.name.is_empty()
            || !self.country.is_empty()
            || !self.language.is_empty()
            || !self.tag.is_empty()
    }

    fn to_search_options(&self) -> (String, SearchOptions<'_>) {
        (
            if self.limit.is_empty() {
                "20".to_string()
            } else {
                self.limit.clone()
            },
            SearchOptions {
                station_name: if self.name.is_empty() {
                    None
                } else {
                    Some(&self.name)
                },
                country: if self.country.is_empty() {
                    None
                } else {
                    Some(&self.country)
                },
                language: if self.language.is_empty() {
                    None
                } else {
                    Some(&self.language)
                },
                tag: if self.tag.is_empty() {
                    None
                } else {
                    Some(&self.tag)
                },
                country_code: None,
                state: None,
                codec: None,
                bitrate_min: None,
                bitrate_max: None,
            },
        )
    }
}

// ── State utama app ──────────────────────────────────────────────────────────
struct App {
    focus: Focus,
    form: SearchForm,
    stations: Vec<ApiStation>,
    list_state: ListState,
    scroll_state: ScrollbarState,
    modal: Modal,
    status_text: String,
    is_loading: bool,
    playing: Option<String>,
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);
        Self {
            focus: Focus::Search,
            form: SearchForm::new(),
            stations: Vec::new(),
            list_state,
            scroll_state: ScrollbarState::default(),
            modal: Modal::None,
            status_text: "Tekan / untuk mulai pencarian — F1 untuk bantuan".to_string(),
            is_loading: false,
            playing: None,
        }
    }

    fn selected_station(&self) -> Option<&ApiStation> {
        self.list_state
            .selected()
            .and_then(|i| self.stations.get(i))
    }

    fn select_next(&mut self) {
        if self.stations.is_empty() {
            return;
        }
        let next = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.stations.len() - 1),
            None => 0,
        };
        self.list_state.select(Some(next));
        self.scroll_state = self.scroll_state.position(next);
    }

    fn select_prev(&mut self) {
        if self.stations.is_empty() {
            return;
        }
        let prev = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(prev));
        self.scroll_state = self.scroll_state.position(prev);
    }

    fn select_first(&mut self) {
        if !self.stations.is_empty() {
            self.list_state.select(Some(0));
            self.scroll_state = self.scroll_state.position(0);
        }
    }

    fn select_last(&mut self) {
        if !self.stations.is_empty() {
            let last = self.stations.len() - 1;
            self.list_state.select(Some(last));
            self.scroll_state = self.scroll_state.position(last);
        }
    }

    fn do_search(&mut self) {
        if !self.form.has_filter() {
            self.status_text = "⚠ Isi minimal satu filter sebelum mencari".to_string();
            return;
        }
        self.is_loading = true;
        self.status_text = "⏳ Mencari stasiun...".to_string();

        let (limit, opts) = self.form.to_search_options();

        match RadioBrowserApp::new(false) {
            Ok(app) => match app.search_builder(&limit, opts) {
                Ok(results) => {
                    let n = results.len();
                    self.stations = results;
                    self.scroll_state = ScrollbarState::new(self.stations.len());
                    if n == 0 {
                        self.status_text = "Tidak ada stasiun ditemukan".to_string();
                        self.list_state.select(None);
                    } else {
                        self.status_text = format!("✓ Ditemukan {} stasiun", n);
                        self.list_state.select(Some(0));
                        self.scroll_state = self.scroll_state.position(0);
                        self.focus = Focus::Results;
                    }
                }
                Err(e) => {
                    self.modal = Modal::Error(format!("Pencarian gagal:\n{e}"));
                    self.status_text = "✗ Pencarian gagal".to_string();
                }
            },
            Err(e) => {
                self.modal = Modal::Error(format!("Koneksi gagal:\n{e}"));
                self.status_text = "✗ Koneksi gagal".to_string();
            }
        }
        self.is_loading = false;
    }

    fn do_play(&mut self) {
        if let Some(station) = self.selected_station() {
            let name = station.name.clone();
            let url = station.url.clone();
            self.playing = Some(name.clone());
            self.modal = Modal::Playing(name.clone());
            self.status_text = format!("▶ Memutar: {}", name);

            // Jalankan mpv di background thread
            std::thread::spawn(move || {
                let _ = crate::play_url(&url);
            });
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────
pub fn tui_main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| render(frame, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Modal aktif → tangani dulu
            if app.modal != Modal::None {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                        app.modal = Modal::None;
                    }
                    _ => {}
                }
                continue;
            }

            // Shortcut global
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Char('q') if app.focus == Focus::Results => return Ok(()),
                KeyCode::F(1) => {
                    app.modal = Modal::Help;
                    continue;
                }
                _ => {}
            }

            // Per-focus handling
            match app.focus {
                Focus::Search => handle_search_input(&mut app, key.code),
                Focus::Results => handle_results_input(&mut app, key.code),
            }
        }
    }
}

fn handle_search_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.do_search(),
        KeyCode::Tab => app.form.next_field(),
        KeyCode::BackTab => app.form.prev_field(),
        KeyCode::Esc => {
            if !app.stations.is_empty() {
                app.focus = Focus::Results;
            }
        }
        KeyCode::Down => {
            if !app.stations.is_empty() {
                app.focus = Focus::Results;
            } else {
                app.form.next_field();
            }
        }
        KeyCode::Left => {
            if app.form.cursor_pos > 0 {
                app.form.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            let len = app.form.active_value().len();
            if app.form.cursor_pos < len {
                app.form.cursor_pos += 1;
            }
        }
        KeyCode::Home => app.form.cursor_pos = 0,
        KeyCode::End => {
            let len = app.form.active_value().len();
            app.form.cursor_pos = len;
        }
        KeyCode::Backspace => app.form.pop_char(),
        KeyCode::Delete => app.form.delete_char(),
        KeyCode::Char(c) => app.form.push_char(c),
        _ => {}
    }
}

fn handle_results_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Up if app.list_state.selected() == Some(0) => {
            app.focus = Focus::Search;
        }
        KeyCode::Up => app.select_prev(),
        KeyCode::Down => app.select_next(),
        KeyCode::Char('k') => app.select_prev(),
        KeyCode::Char('j') => app.select_next(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Enter | KeyCode::Char('p') => app.do_play(),
        KeyCode::Char('/') => {
            app.focus = Focus::Search;
        }
        _ => {}
    }
}

// ── Render utama ─────────────────────────────────────────────────────────────
fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

    // Layout: header, body (search | results + detail), statusbar
    let root = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(10),   // body
        Constraint::Length(1), // statusbar
    ])
    .split(area);

    render_header(frame, root[0]);

    let body =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).split(root[1]);

    render_search_panel(frame, app, body[0]);

    let right =
        Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(body[1]);

    render_results_panel(frame, app, right[0]);
    render_detail_panel(frame, app, right[1]);

    render_statusbar(frame, app, root[2]);

    // Modal overlay
    match &app.modal {
        Modal::Help => render_help_modal(frame, area),
        Modal::Error(msg) => render_error_modal(frame, area, msg.clone()),
        Modal::Playing(name) => render_playing_modal(frame, area, name.clone()),
        Modal::None => {}
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG2));

    let title_spans = Line::from(vec![
        Span::styled("  📻 ", Style::default()),
        Span::styled(
            "REDIO",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  radio browser", Style::default().fg(FG_DIM)),
        Span::styled(
            "  [F1 Bantuan]  [Ctrl+C Keluar]",
            Style::default().fg(FG_DIM),
        ),
    ]);

    let para = Paragraph::new(title_spans)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(para, area);
}

fn render_search_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focus = app.focus == Focus::Search;
    let border_color = if is_focus { BORDER_FOCUS } else { BORDER };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" 🔍 ", Style::default()),
            Span::styled(
                "Pencarian",
                Style::default().fg(FG).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG2));

    frame.render_widget(block, area);

    // Inner area
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let fields = [
        ("Nama Stasiun", &app.form.name, 0usize),
        ("Negara", &app.form.country, 1),
        ("Bahasa", &app.form.language, 2),
        ("Tag/Genre", &app.form.tag, 3),
        ("Limit", &app.form.limit, 4),
    ];

    let field_height = 3u16;
    let gap = 0u16;

    for (i, (label, value, idx)) in fields.iter().enumerate() {
        let y = inner.y + (i as u16) * (field_height + gap);
        if y + field_height > area.y + area.height {
            break;
        }

        let field_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: field_height,
        };

        let is_active = is_focus && app.form.active_field == *idx;
        let label_color = if is_active { ACCENT } else { FG_DIM };
        let value_color = if is_active {
            FG
        } else {
            Color::Rgb(180, 178, 170)
        };
        let bg_color = if is_active { BG3 } else { BG2 };

        let text = Text::from(vec![
            Line::from(Span::styled(
                *label,
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                if value.is_empty() {
                    "─".repeat(inner.width as usize / 2)
                } else {
                    (*value).clone()
                },
                Style::default()
                    .fg(if value.is_empty() {
                        BORDER
                    } else {
                        value_color
                    })
                    .bg(bg_color),
            )),
        ]);

        let field_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(if is_active { ACCENT } else { BORDER }));

        let para = Paragraph::new(text).block(field_block);
        frame.render_widget(para, field_area);

        // Cursor
        if is_active {
            let cursor_x = field_area.x + app.form.cursor_pos.min(value.len()) as u16;
            let cursor_y = field_area.y + 1;
            if cursor_x < field_area.x + field_area.width {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }

    // Tombol Search
    let btn_y = inner.y + 5 * (field_height + gap) + 1;
    if btn_y + 1 < area.y + area.height {
        let has = app.form.has_filter();
        let (btn_text, btn_fg, btn_bg) = if has {
            (" ↵  CARI SEKARANG  ↵ ", FG, ACCENT)
        } else {
            (" Isi filter untuk mencari  ", FG_DIM, BG3)
        };

        let btn_area = Rect {
            x: inner.x,
            y: btn_y,
            width: inner.width,
            height: 1,
        };

        let btn = Paragraph::new(btn_text)
            .style(
                Style::default()
                    .fg(btn_fg)
                    .bg(btn_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(btn, btn_area);
    }
}

fn render_results_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focus = app.focus == Focus::Results;
    let border_color = if is_focus { BORDER_FOCUS } else { BORDER };
    let count = app.stations.len();

    let title_text = if count == 0 {
        " 📋 Hasil Pencarian ".to_string()
    } else {
        format!(" 📋 Hasil Pencarian  ({} stasiun) ", count)
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            title_text,
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG));

    if app.stations.is_empty() {
        let empty_text = if app.is_loading {
            "⏳ Sedang mencari..."
        } else {
            "Belum ada hasil. Gunakan panel pencarian\ndi sebelah kiri untuk mencari stasiun."
        };
        let para = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(FG_DIM))
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .stations
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_playing = app.playing.as_deref() == Some(s.name.as_str());
            let num = format!("{:>3}. ", i + 1);
            let bitrate = if s.bitrate == 0 {
                String::new()
            } else {
                format!("{}kbps", s.bitrate)
            };
            let codec = if s.codec.is_empty() {
                "?".to_string()
            } else {
                s.codec.to_uppercase()
            };
            let right = format!(" {} {} ", codec, bitrate);

            let name_color = if is_playing { GREEN } else { FG };
            let prefix = if is_playing { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(num, Style::default().fg(FG_DIM)),
                Span::styled(prefix, Style::default().fg(GREEN)),
                Span::styled(s.name.clone(), Style::default().fg(name_color)),
                Span::styled(right, Style::default().fg(FG_DIM)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(BG3).fg(FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    let mut list_state = app.list_state;
    frame.render_stateful_widget(list, area, &mut list_state);
    app.list_state = list_state;

    // Scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .style(Style::default().fg(BORDER));

    let mut scroll_state = app.scroll_state.content_length(app.stations.len());
    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            horizontal: 0,
            vertical: 1,
        }),
        &mut scroll_state,
    );
    app.scroll_state = scroll_state;
}

fn render_detail_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " ℹ Detail Stasiun",
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(BG2));

    let Some(station) = app.selected_station() else {
        let para = Paragraph::new("Pilih stasiun dari daftar hasil\nuntuk melihat detail")
            .block(block)
            .alignment(Alignment::Center)
            .style(Style::default().fg(FG_DIM));
        frame.render_widget(para, area);
        return;
    };

    let name = station.name.clone();
    let country = if station.country.is_empty() {
        "-".to_string()
    } else {
        station.country.clone()
    };
    let language = if station.language.is_empty() {
        "-".to_string()
    } else {
        station.language.clone()
    };
    let codec = if station.codec.is_empty() {
        "-".to_string()
    } else {
        station.codec.to_uppercase()
    };
    let bitrate = if station.bitrate == 0 {
        "-".to_string()
    } else {
        format!("{} kbps", station.bitrate)
    };
    let url = station.url.chars().take(48).collect::<String>();
    let tags = if station.tags.is_empty() {
        "-"
    } else {
        station.tags.as_str()
    };
    let votes = station.votes.to_string();
    let clicks = station.clickcount.to_string();

    let is_playing = app.playing.as_deref() == Some(name.as_str());
    let play_hint = if is_playing {
        Line::from(Span::styled(
            " ▶ SEDANG DIPUTAR",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::styled(
            " [Enter / p] untuk memutar",
            Style::default().fg(FG_DIM),
        ))
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Nama     ", Style::default().fg(FG_DIM)),
            Span::styled(
                name,
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Negara   ", Style::default().fg(FG_DIM)),
            Span::styled(country, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Bahasa   ", Style::default().fg(FG_DIM)),
            Span::styled(language, Style::default().fg(FG)),
        ]),
        Line::from(vec![
            Span::styled("  Codec    ", Style::default().fg(FG_DIM)),
            Span::styled(codec, Style::default().fg(AMBER)),
        ]),
        Line::from(vec![
            Span::styled("  Bitrate  ", Style::default().fg(FG_DIM)),
            Span::styled(bitrate, Style::default().fg(AMBER)),
        ]),
        Line::from(vec![
            Span::styled("  URL      ", Style::default().fg(FG_DIM)),
            Span::styled(url, Style::default().fg(Color::Rgb(120, 180, 220))),
        ]),
        Line::from(vec![
            Span::styled("  Tags     ", Style::default().fg(FG_DIM)),
            Span::styled(
                tags.chars().take(40).collect::<String>(),
                Style::default().fg(FG_DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Votes    ", Style::default().fg(FG_DIM)),
            Span::styled(votes, Style::default().fg(GREEN)),
            Span::styled("   Klik ", Style::default().fg(FG_DIM)),
            Span::styled(clicks, Style::default().fg(GREEN)),
        ]),
        Line::from(""),
        play_hint,
    ];

    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

fn render_statusbar(frame: &mut Frame, app: &mut App, area: Rect) {
    let hints = match app.focus {
        Focus::Search => " Tab: pindah field  ↵: cari  ↓/Esc: ke hasil  F1: bantuan ",
        Focus::Results => " ↑↓/jk: navigasi  ↵/p: putar  /: cari lagi  q: keluar ",
    };

    let status_color = if app.status_text.starts_with('✓') {
        GREEN
    } else if app.status_text.starts_with('✗') || app.status_text.starts_with('⚠') {
        ACCENT
    } else {
        FG_DIM
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", app.status_text),
            Style::default().fg(status_color).bg(BG2),
        ),
        Span::styled(hints, Style::default().fg(FG_DIM).bg(BG)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

// ── Modal ─────────────────────────────────────────────────────────────────────
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    let modal_area = centered_rect(58, 22, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " 📖 Bantuan ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG2));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  PANEL PENCARIAN",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Tab / Shift+Tab   pindah antar field"),
        Line::from("  ← →               geser kursor"),
        Line::from("  Enter             mulai pencarian"),
        Line::from("  Esc / ↓           ke panel hasil"),
        Line::from(""),
        Line::from(Span::styled(
            "  PANEL HASIL",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  ↑ ↓  /  k j       navigasi daftar"),
        Line::from("  g / G             lompat ke atas/bawah"),
        Line::from("  Enter / p         putar stasiun via mpv"),
        Line::from("  /                 kembali ke pencarian"),
        Line::from("  q                 keluar"),
        Line::from(""),
        Line::from(Span::styled(
            "  GLOBAL",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  F1                buka bantuan ini"),
        Line::from("  Ctrl+C            keluar kapanpun"),
        Line::from(""),
        Line::from(Span::styled(
            "  [Esc / Enter / q] tutup",
            Style::default().fg(FG_DIM),
        )),
    ];

    let para = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(FG));
    frame.render_widget(para, modal_area);
}

fn render_error_modal(frame: &mut Frame, area: Rect, msg: String) {
    let modal_area = centered_rect(50, 10, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " ✗ Error ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG2));

    let lines: Vec<Line> = std::iter::once(Line::from(""))
        .chain(msg.lines().map(|l| Line::from(format!("  {l}"))))
        .chain([
            Line::from(""),
            Line::from(Span::styled(
                "  [Esc / Enter] tutup",
                Style::default().fg(FG_DIM),
            )),
        ])
        .collect();

    let para = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(FG))
        .wrap(Wrap { trim: true });
    frame.render_widget(para, modal_area);
}

fn render_playing_modal(frame: &mut Frame, area: Rect, name: String) {
    let modal_area = centered_rect(50, 8, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(Span::styled(
            " ▶ Memutar ",
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(GREEN))
        .style(Style::default().bg(BG2));

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ♫  {}", name),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  mpv sedang berjalan di background.",
            Style::default().fg(FG_DIM),
        )),
        Line::from(Span::styled(
            "  Tutup jendela terminal untuk stop.",
            Style::default().fg(FG_DIM),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [Esc / Enter] tutup pesan ini",
            Style::default().fg(FG_DIM),
        )),
    ];

    let para = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(FG));
    frame.render_widget(para, modal_area);
}
