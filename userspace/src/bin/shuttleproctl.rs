use clap::{Parser, Subcommand};
use crossterm::event::{self, Event as TerminalEvent, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Terminal;
use shuttlepro::config::{CompiledProfile, Profile};
use shuttlepro::device;
use shuttlepro::input::{EventDevice, InputEvent, ABS_MISC, EV_ABS, EV_KEY, EV_REL, REL_DIAL};
use shuttlepro::keys::KeyChord;
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const BUTTON_BASE: u16 = 704;
const BUTTON_COUNT: usize = 13;
const EVENT_LOG_LEN: usize = 12;

#[derive(Parser)]
#[command(
    version,
    about = "Test and inspect Contour ShuttlePro v2 userspace profiles"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Detect {
        #[arg(long, default_value_t = 0x0b33)]
        vendor: u16,
        #[arg(long, default_value_t = 0x0030)]
        product: u16,
        #[arg(long, default_value = "Contour ShuttlePro v2")]
        name: String,
    },
    Monitor {
        #[arg(long)]
        event: Option<PathBuf>,
    },
    Tui {
        #[arg(long)]
        event: Option<PathBuf>,
        #[arg(long, default_value_t = 60)]
        fps: u16,
    },
    Keymap {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        title: Option<String>,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommand,
    },
}

#[derive(Subcommand)]
enum ProfileCommand {
    Validate { file: PathBuf },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Detect {
            vendor,
            product,
            name,
        } => {
            let Some(device) = device::find(vendor, product, &name)? else {
                return Err("ShuttlePro event device not found".into());
            };

            println!("{}", device.event.display());
        }
        Command::Monitor { event } => {
            let event = match event {
                Some(path) => path,
                None => {
                    device::find(0x0b33, 0x0030, "Contour ShuttlePro v2")?
                        .ok_or("ShuttlePro event device not found")?
                        .event
                }
            };
            let device = EventDevice::open(&event, false)?;

            loop {
                if let Some(input) = device.read_event()? {
                    print_event(input.event_type, input.code, input.value);
                }
            }
        }
        Command::Tui { event, fps } => {
            let event = match event {
                Some(path) => path,
                None => {
                    device::find(0x0b33, 0x0030, "Contour ShuttlePro v2")?
                        .ok_or("ShuttlePro event device not found")?
                        .event
                }
            };

            run_tui(event, fps.max(1))?;
        }
        Command::Keymap {
            profile,
            output,
            title,
        } => {
            let profile = Profile::load(&profile)?.compile()?;
            let title =
                title.unwrap_or_else(|| format!("{} ShuttlePro v2 Keymap", profile.profile.name));
            let svg = render_keymap_svg(&profile, &title);

            fs::write(&output, svg)?;
            println!("wrote {}", output.display());
        }
        Command::Profile {
            command: ProfileCommand::Validate { file },
        } => {
            let profile = Profile::load(&file)?.compile()?;
            println!("valid profile: {}", profile.profile.name);
        }
    }

    Ok(())
}

fn render_keymap_svg(profile: &CompiledProfile, title: &str) -> String {
    let mut svg = String::new();

    svg.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    svg.push('\n');
    svg.push_str(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="11in" height="8.5in" viewBox="0 0 1100 850">"#,
    );
    svg.push('\n');
    svg.push_str(
        r#"<style>
text { font-family: "Inter", "DejaVu Sans", Arial, sans-serif; fill: #111827; }
.small { font-size: 18px; }
.action { font-size: 16px; font-weight: 700; }
.shortcut { font-size: 15px; fill: #4b5563; }
.label { font-size: 22px; font-weight: 700; }
.title { font-size: 36px; font-weight: 800; }
.meta { font-size: 16px; fill: #4b5563; }
.button { fill: #f9fafb; stroke: #111827; stroke-width: 2; }
.button-num { font-size: 16px; font-weight: 800; fill: #6b7280; }
.control { fill: #eef2ff; stroke: #3730a3; stroke-width: 2; }
.outline { fill: #ffffff; stroke: #111827; stroke-width: 3; }
.cut { fill: none; stroke: #9ca3af; stroke-width: 1; stroke-dasharray: 8 8; }
</style>"#,
    );
    svg.push('\n');

    rect(&mut svg, 35, 35, 1030, 780, 28, "outline");
    line(&mut svg, 70, 760, 1030, 760, "cut");
    text(&mut svg, 70, 88, "title", title);
    text(
        &mut svg,
        70,
        118,
        "meta",
        "Generated from profile TOML. Print at 100% scale; verify fit before cutting or laminating.",
    );

    draw_button_grid(&mut svg, profile);
    draw_transport_controls(&mut svg, profile);
    draw_legend(&mut svg);

    svg.push_str("</svg>\n");
    svg
}

fn draw_button_grid(svg: &mut String, profile: &CompiledProfile) {
    let buttons = [
        (1, 95, 165),
        (2, 235, 165),
        (3, 375, 165),
        (4, 515, 165),
        (5, 655, 165),
        (6, 95, 285),
        (7, 235, 285),
        (8, 375, 285),
        (9, 515, 285),
        (10, 655, 285),
        (11, 235, 405),
        (12, 375, 405),
        (13, 515, 405),
    ];

    for (number, x, y) in buttons {
        draw_button(svg, number, x, y, button_action(profile, number));
    }
}

fn draw_transport_controls(svg: &mut String, profile: &CompiledProfile) {
    circle(svg, 890, 265, 95, "control");
    text_center(svg, 890, 245, "label", "Jog");
    text_center(
        svg,
        890,
        276,
        "small",
        &format!(
            "CW: {} ({})",
            label_or(&profile.jog.positive_label, "Jog Right"),
            chords(&profile.jog.positive)
        ),
    );
    text_center(
        svg,
        890,
        305,
        "small",
        &format!(
            "CCW: {} ({})",
            label_or(&profile.jog.negative_label, "Jog Left"),
            chords(&profile.jog.negative)
        ),
    );

    rect(svg, 765, 420, 250, 95, 44, "control");
    text_center(svg, 890, 455, "label", "Shuttle Ring");
    text_center(
        svg,
        890,
        486,
        "small",
        &format!(
            "- {} ({})",
            label_or(&profile.shuttle.negative_label, "Reverse"),
            chords(&profile.shuttle.negative)
        ),
    );
    text_center(
        svg,
        890,
        512,
        "small",
        &format!(
            "0 {} ({})",
            label_or(&profile.shuttle.neutral_label, "Pause"),
            chords(&profile.shuttle.neutral)
        ),
    );
    text_center(
        svg,
        890,
        538,
        "small",
        &format!(
            "+ {} ({})",
            label_or(&profile.shuttle.positive_label, "Forward"),
            chords(&profile.shuttle.positive)
        ),
    );
}

fn draw_legend(svg: &mut String) {
    text(
        svg,
        80,
        775,
        "meta",
        "Button numbers follow driver order BTN_TRIGGER_HAPPY1..13.",
    );
    text(
        svg,
        80,
        800,
        "meta",
        "This is a printable keymap/reference sheet, not a dimension-certified adhesive overlay.",
    );
}

fn draw_button(svg: &mut String, number: u8, x: i32, y: i32, action: String) {
    rect(svg, x, y, 108, 76, 12, "button");
    text(svg, x + 10, y + 24, "button-num", &format!("{number:02}"));
    let (label, shortcut) = button_label_and_shortcut(&action);
    wrapped_text(svg, x + 14, y + 47, 84, "action", &label);
    wrapped_text(svg, x + 14, y + 69, 84, "shortcut", &shortcut);
}

fn button_action(profile: &CompiledProfile, number: u8) -> String {
    let Some(button) = profile.buttons.get(&number) else {
        return "-|-".to_string();
    };
    let shortcut = chords(&button.press);
    let label = button
        .label
        .clone()
        .filter(|label| !label.is_empty())
        .unwrap_or_else(|| shortcut.clone());

    format!("{label}|{shortcut}")
}

fn chords(chords: &[KeyChord]) -> String {
    chords
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn button_label_and_shortcut(action: &str) -> (String, String) {
    let Some((label, shortcut)) = action.split_once('|') else {
        return (action.to_string(), String::new());
    };

    (label.to_string(), shortcut.to_string())
}

fn label_or<'a>(label: &'a Option<String>, fallback: &'a str) -> &'a str {
    label
        .as_deref()
        .filter(|label| !label.is_empty())
        .unwrap_or(fallback)
}

fn wrapped_text(svg: &mut String, x: i32, y: i32, width: usize, class: &str, text_value: &str) {
    let max_chars = (width / 9).max(6);
    let mut line = String::new();
    let mut dy = 0;

    for word in text_value.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > max_chars {
            text(svg, x, y + dy, class, &line);
            line.clear();
            dy += 21;
        }

        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }

    if !line.is_empty() {
        text(svg, x, y + dy, class, &line);
    }
}

fn rect(svg: &mut String, x: i32, y: i32, width: i32, height: i32, radius: i32, class: &str) {
    let _ = writeln!(
        svg,
        r#"<rect class="{class}" x="{x}" y="{y}" width="{width}" height="{height}" rx="{radius}"/>"#
    );
}

fn circle(svg: &mut String, cx: i32, cy: i32, radius: i32, class: &str) {
    let _ = writeln!(
        svg,
        r#"<circle class="{class}" cx="{cx}" cy="{cy}" r="{radius}"/>"#
    );
}

fn line(svg: &mut String, x1: i32, y1: i32, x2: i32, y2: i32, class: &str) {
    let _ = writeln!(
        svg,
        r#"<line class="{class}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}"/>"#
    );
}

fn text(svg: &mut String, x: i32, y: i32, class: &str, text_value: &str) {
    let _ = writeln!(
        svg,
        r#"<text class="{class}" x="{x}" y="{y}">{}</text>"#,
        escape_xml(text_value)
    );
}

fn text_center(svg: &mut String, x: i32, y: i32, class: &str, text_value: &str) {
    let _ = writeln!(
        svg,
        r#"<text class="{class}" x="{x}" y="{y}" text-anchor="middle">{}</text>"#,
        escape_xml(text_value)
    );
}

fn escape_xml(text_value: &str) -> String {
    text_value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Debug)]
struct TuiState {
    event_path: PathBuf,
    buttons: [bool; BUTTON_COUNT],
    shuttle: i32,
    jog_total: i64,
    jog_last: i32,
    event_count: u64,
    last_event_at: Option<Instant>,
    events: VecDeque<String>,
}

impl TuiState {
    fn new(event_path: PathBuf) -> Self {
        Self {
            event_path,
            buttons: [false; BUTTON_COUNT],
            shuttle: 0,
            jog_total: 0,
            jog_last: 0,
            event_count: 0,
            last_event_at: None,
            events: VecDeque::with_capacity(EVENT_LOG_LEN),
        }
    }

    fn apply(&mut self, event: InputEvent) {
        let Some(label) = self.describe(event) else {
            return;
        };

        self.event_count += 1;
        self.last_event_at = Some(Instant::now());

        if self.events.len() == EVENT_LOG_LEN {
            self.events.pop_front();
        }
        self.events.push_back(label);
    }

    fn describe(&mut self, event: InputEvent) -> Option<String> {
        match event.event_type {
            EV_KEY => {
                let button = event.code.checked_sub(BUTTON_BASE)? as usize;
                if button >= BUTTON_COUNT {
                    return None;
                }

                self.buttons[button] = event.value != 0;
                let state = if event.value != 0 {
                    "pressed"
                } else {
                    "released"
                };
                Some(format!("button {:02} {}", button + 1, state))
            }
            EV_REL if event.code == REL_DIAL => {
                self.jog_last = event.value;
                self.jog_total += i64::from(event.value);
                Some(format!("jog delta {:+}", event.value))
            }
            EV_ABS if event.code == ABS_MISC => {
                self.shuttle = event.value.clamp(-7, 7);
                Some(format!("shuttle {:+}", self.shuttle))
            }
            _ => None,
        }
    }
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        Ok(Self { terminal })
    }

    fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn run_tui(event_path: PathBuf, fps: u16) -> Result<(), Box<dyn std::error::Error>> {
    let device = EventDevice::open(&event_path, true)?;
    let mut state = TuiState::new(event_path);
    let mut guard = TerminalGuard::new()?;
    let frame_interval = Duration::from_millis(1000 / u64::from(fps));
    let shutdown = Arc::new(AtomicBool::new(false));

    flag::register(SIGINT, Arc::clone(&shutdown))?;
    flag::register(SIGTERM, Arc::clone(&shutdown))?;

    while !shutdown.load(Ordering::Relaxed) {
        while let Some(input) = device.read_event()? {
            state.apply(input);
        }

        guard.terminal().draw(|frame| draw(frame, &state))?;

        if event::poll(frame_interval)? && should_exit(event::read()?) {
            break;
        }
    }

    Ok(())
}

fn should_exit(event: TerminalEvent) -> bool {
    match event {
        TerminalEvent::Key(key) => {
            matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        }
        _ => false,
    }
}

fn draw(frame: &mut ratatui::Frame<'_>, state: &TuiState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(8),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_shuttle(frame, chunks[1], state);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(chunks[2]);
    draw_buttons(frame, middle[0], state);
    draw_jog(frame, middle[1], state);
    draw_events(frame, chunks[3], state);
}

fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let age = state
        .last_event_at
        .map(|instant| format!("{} ms ago", instant.elapsed().as_millis()))
        .unwrap_or_else(|| "waiting".to_string());
    let text = vec![Line::from(vec![
        Span::styled("ShuttlePro v2", Style::new().bold().fg(Color::Cyan)),
        Span::raw("  "),
        Span::raw(state.event_path.display().to_string()),
        Span::raw("  |  events: "),
        Span::styled(state.event_count.to_string(), Style::new().fg(Color::Green)),
        Span::raw("  |  last: "),
        Span::styled(age, Style::new().fg(Color::Yellow)),
        Span::raw("  |  q/Esc exits"),
    ])];

    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Status")),
        area,
    );
}

fn draw_shuttle(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let ratio = ((state.shuttle + 7) as f64 / 14.0).clamp(0.0, 1.0);
    let label = format!(
        "shuttle {:+}  {}",
        state.shuttle,
        shuttle_bar(state.shuttle)
    );
    let style = if state.shuttle < 0 {
        Style::new().fg(Color::Red)
    } else if state.shuttle > 0 {
        Style::new().fg(Color::Green)
    } else {
        Style::new().fg(Color::Gray)
    };

    frame.render_widget(
        Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Shuttle Ring"))
            .gauge_style(style)
            .ratio(ratio)
            .label(label),
        area,
    );
}

fn draw_buttons(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let rows = state
        .buttons
        .chunks(5)
        .enumerate()
        .map(|(row, buttons)| {
            let spans = buttons
                .iter()
                .enumerate()
                .flat_map(|(col, pressed)| {
                    let number = row * 5 + col + 1;
                    let style = if *pressed {
                        Style::new()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::new().fg(Color::DarkGray)
                    };

                    [
                        Span::styled(format!(" {:02} ", number), style),
                        Span::raw(" "),
                    ]
                })
                .collect::<Vec<_>>();

            Line::from(spans)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(rows)
            .block(Block::default().borders(Borders::ALL).title("Buttons"))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_jog(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let direction = if state.jog_last > 0 {
        "clockwise"
    } else if state.jog_last < 0 {
        "counter-clockwise"
    } else {
        "idle"
    };
    let text = vec![
        Line::from(vec![
            Span::raw("last: "),
            Span::styled(
                format!("{:+}", state.jog_last),
                Style::new().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("total: "),
            Span::styled(state.jog_total.to_string(), Style::new().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("direction: "),
            Span::styled(direction, Style::new().fg(Color::Green)),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Jog Wheel"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_events(frame: &mut ratatui::Frame<'_>, area: Rect, state: &TuiState) {
    let items = state
        .events
        .iter()
        .rev()
        .map(|event| ListItem::new(event.as_str()))
        .collect::<Vec<_>>();

    frame.render_widget(
        List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Events"),
            )
            .highlight_style(Style::new().fg(Color::Yellow)),
        area,
    );
}

fn shuttle_bar(value: i32) -> String {
    (-7..=7)
        .map(|level| {
            if level == value {
                'O'
            } else if level == 0 {
                '+'
            } else {
                '-'
            }
        })
        .collect()
}

fn print_event(event_type: u16, code: u16, value: i32) {
    match event_type {
        EV_KEY => println!("button code={} value={}", code, value),
        EV_REL if code == REL_DIAL => println!("jog delta={}", value),
        EV_ABS if code == ABS_MISC => println!("shuttle value={}", value),
        _ => {}
    }
}
