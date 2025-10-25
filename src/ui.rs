use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::app::{App, AppState};
use crate::game::{LightState, ShiftQuality};

pub fn draw(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Menu => draw_menu(f, app),
        AppState::Racing => {
            if let Some(race) = &app.race_state {
                draw_race(f, race);
            }
        }
        AppState::Results => {
            if let Some(race) = &app.race_state {
                draw_results(f, race, app);
            }
        }
    }
}

fn draw_menu(f: &mut Frame, app: &App) {
    let area = f.area();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "DRAG-RS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Terminal Drag Racing"),
        Line::from(""),
        Line::from(format!("Selected Car: {}", app.player_car.name)),
        Line::from(format!(
            "Horsepower: {} | Weight: {}kg | Redline: {} RPM",
            app.player_car.horsepower, app.player_car.weight, app.player_car.redline
        )),
        Line::from(""),
        Line::from(format!(
            "Audio: {}",
            if app.audio_muted {
                Span::styled("MUTED", Style::default().fg(Color::Red))
            } else {
                Span::styled("ON", Style::default().fg(Color::Green))
            }
        )),
        Line::from(""),
        Line::from("[←/→] Select Car"),
        Line::from("[1] Quick Race"),
        Line::from("[M] Toggle Audio"),
        Line::from("[Q] Quit"),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"));

    f.render_widget(paragraph, area);
}

fn draw_race(f: &mut Frame, race: &crate::game::RaceState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(8),  // Christmas tree & track
            Constraint::Length(10), // Cars on track
            Constraint::Min(8),     // Gauges
            Constraint::Length(3),  // Controls
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(format!("DRAG-RS | ET: {:.3}s", race.elapsed_time))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Christmas tree and distance
    draw_christmas_tree(f, chunks[1], race);

    // Track visualization
    draw_track(f, chunks[2], race);

    // Gauges
    draw_gauges(f, chunks[3], race);

    // Controls
    let controls =
        Paragraph::new("[SPACE] Throttle | [↑/W] Shift | [N] Nitrous | [M] Mute | [ESC] Menu")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[4]);
}

fn draw_christmas_tree(f: &mut Frame, area: Rect, race: &crate::game::RaceState) {
    let tree_text = match race.christmas_tree.state {
        LightState::PreStage => vec![
            Line::from("  ⚪ Pre-Stage"),
            Line::from("  ⚫ Staged"),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from(""),
        ],
        LightState::Staged => vec![
            Line::from("  ⚪ Pre-Stage"),
            Line::from("  ⚪ Staged"),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from(""),
        ],
        LightState::Yellow1 => vec![
            Line::from("  ⚪"),
            Line::from("  ⚪"),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from(""),
        ],
        LightState::Yellow2 => vec![
            Line::from("  ⚪"),
            Line::from("  ⚪"),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from("  ⚫"),
            Line::from(""),
        ],
        LightState::Yellow3 => vec![
            Line::from("  ⚪"),
            Line::from("  ⚪"),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from(Span::styled("  🟡", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ],
        LightState::Green => vec![
            Line::from("  ⚪"),
            Line::from("  ⚪"),
            Line::from(Span::styled(
                "  🟢 GO!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  ⚫"),
            Line::from("  ⚫"),
            Line::from(""),
        ],
        LightState::Racing => {
            if let Some(rt) = race.player.reaction_time {
                if rt < 0.0 {
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  🔴 RED LIGHT!",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )),
                        Line::from(""),
                        Line::from("  You jumped the start!"),
                        Line::from(""),
                        Line::from(""),
                    ]
                } else {
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  RACING!",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(""),
                        Line::from(format!("  Distance: {:.1}m / 402.3m", race.player.position)),
                        Line::from(""),
                        Line::from(""),
                    ]
                }
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  RACING!",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(format!("  Distance: {:.1}m / 402.3m", race.player.position)),
                    Line::from(""),
                    Line::from(""),
                ]
            }
        }
    };

    let tree = Paragraph::new(tree_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).title("Start"));
    f.render_widget(tree, area);
}

fn draw_track(f: &mut Frame, area: Rect, race: &crate::game::RaceState) {
    // Ensure track fits within terminal width with proper margins
    let track_width = (area.width as usize).saturating_sub(10).max(20);

    // Calculate positions (clamp to track width)
    let player_progress = race.get_player_progress().min(1.0);
    let opponent_progress = race.get_opponent_progress().min(1.0);
    let player_pos = (player_progress * (track_width - 1) as f64) as usize;
    let opponent_pos = (opponent_progress * (track_width - 1) as f64) as usize;

    // Build track lines
    let mut opponent_line = String::from("Opp: ");
    let mut player_line = String::from("You: ");

    for i in 0..track_width {
        // Opponent line
        if i == opponent_pos {
            opponent_line.push('▶');
        } else if i == track_width - 1 {
            opponent_line.push('║'); // Finish line
        } else {
            opponent_line.push('─');
        }

        // Player line
        if i == player_pos {
            player_line.push('▶');
        } else if i == track_width - 1 {
            player_line.push('║'); // Finish line
        } else {
            player_line.push('─');
        }
    }

    // Create colored track lines with only the car symbols colored
    let opponent_spans = create_colored_track_line(&opponent_line, Color::Red);
    let player_spans = create_colored_track_line(&player_line, Color::Green);

    let track_text = vec![
        Line::from(""),
        Line::from(format!(
            "Opponent: {:.1} m/s | ET: {:.3}s{}",
            race.opponent.velocity,
            race.opponent.reaction_time.unwrap_or(0.0) + race.elapsed_time,
            if race.opponent.finish_time.is_some() {
                " ✅"
            } else {
                ""
            }
        )),
        Line::from(opponent_spans),
        Line::from(""),
        Line::from(format!(
            "Player: {:.1} m/s | ET: {:.3}s{}",
            race.player.velocity,
            race.player.reaction_time.unwrap_or(0.0) + race.elapsed_time,
            if race.player.finish_time.is_some() {
                " ✅"
            } else {
                ""
            }
        )),
        Line::from(player_spans),
        Line::from(""),
        Line::from(if let Some(quality) = race.player.last_shift_quality {
            match quality {
                ShiftQuality::Perfect => Span::styled(
                    "PERFECT SHIFT! ⚡",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                ShiftQuality::Good => {
                    Span::styled("Good shift", Style::default().fg(Color::Yellow))
                }
                ShiftQuality::Missed => {
                    Span::styled("Missed shift!", Style::default().fg(Color::Red))
                }
                ShiftQuality::TooEarly => {
                    Span::styled("Too early!", Style::default().fg(Color::Red))
                }
            }
        } else {
            Span::raw("")
        }),
    ];

    let track =
        Paragraph::new(track_text).block(Block::default().borders(Borders::ALL).title("Track"));
    f.render_widget(track, area);
}

fn create_colored_track_line(line: &str, car_color: Color) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let mut current_span = String::new();

    for ch in line.chars() {
        if ch == '▶' {
            // Add the current span if it's not empty
            if !current_span.is_empty() {
                spans.push(Span::raw(current_span));
                current_span = String::new();
            }
            // Add the colored car symbol
            spans.push(Span::styled("▶", Style::default().fg(car_color)));
        } else {
            current_span.push(ch);
        }
    }

    // Add any remaining text
    if !current_span.is_empty() {
        spans.push(Span::raw(current_span));
    }

    spans
}

fn draw_gauges(f: &mut Frame, area: Rect, race: &crate::game::RaceState) {
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    // RPM Gauge
    let rpm_percent = (race.player.rpm as f64 / race.player_car.redline as f64 * 100.0) as u16;
    let rpm_color = if rpm_percent > 90 {
        Color::Red
    } else if rpm_percent > 75 {
        Color::Yellow
    } else {
        Color::Green
    };

    let rpm_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            "RPM: {} / {} | Gear: {}",
            race.player.rpm,
            race.player_car.redline,
            race.player.gear + 1
        )))
        .gauge_style(Style::default().fg(rpm_color))
        .percent(rpm_percent.min(100));
    f.render_widget(rpm_gauge, gauge_chunks[0]);

    // NOS Gauge
    let nos_percent = (race.player.nos_remaining / 10.0 * 100.0) as u16;
    let nos_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Nitrous: {:.1}s {}",
            race.player.nos_remaining,
            if race.player.nos_active {
                "🔥 ACTIVE"
            } else {
                ""
            }
        )))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(nos_percent);
    f.render_widget(nos_gauge, gauge_chunks[1]);

    // Heat Gauge
    let heat_percent = (race.player.engine_heat * 100.0) as u16;
    let heat_color = if heat_percent > 80 {
        Color::Red
    } else if heat_percent > 50 {
        Color::Yellow
    } else {
        Color::Green
    };

    let heat_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Engine Heat: {}%{}",
            heat_percent,
            if race.player.blown_engine {
                " 💥 BLOWN!"
            } else {
                ""
            }
        )))
        .gauge_style(Style::default().fg(heat_color))
        .percent(heat_percent);
    f.render_widget(heat_gauge, gauge_chunks[2]);

    // Stats
    let stats_text = vec![
        Line::from(format!(
            "Speed: {:.1} m/s | Top: {:.1} m/s",
            race.player.velocity, race.player.top_speed
        )),
        Line::from(format!(
            "Perfect Shifts: {} | Throttle: {:.0}%",
            race.player.perfect_shifts,
            race.player.throttle * 100.0
        )),
    ];
    let stats =
        Paragraph::new(stats_text).block(Block::default().borders(Borders::ALL).title("Stats"));
    f.render_widget(stats, gauge_chunks[3]);
}

fn draw_results(f: &mut Frame, race: &crate::game::RaceState, app: &App) {
    let area = f.area();

    let (winner_text, winner_color) = if let Some(rt) = race.player.reaction_time {
        if rt < 0.0 {
            ("RED LIGHT! 🔴", Color::Red)
        } else {
            match race.winner {
                Some(crate::game::Winner::Player) => ("YOU WIN! 🏆", Color::Green),
                Some(crate::game::Winner::Opponent) => ("YOU LOSE", Color::Red),
                None => ("DRAW", Color::Yellow),
            }
        }
    } else {
        match race.winner {
            Some(crate::game::Winner::Player) => ("YOU WIN! 🏆", Color::Green),
            Some(crate::game::Winner::Opponent) => ("YOU LOSE", Color::Red),
            None => ("DRAW", Color::Yellow),
        }
    };

    let player_et = race.player.finish_time.unwrap_or(999.0);
    let opponent_et = race.opponent.finish_time.unwrap_or(999.0);
    let player_rt = race.player.reaction_time.unwrap_or(0.0);

    let results_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            winner_text,
            Style::default()
                .fg(winner_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("═══════════════════════════════"),
        Line::from(""),
        Line::from(format!("Your Time:      {:.3}s", player_et)),
        Line::from(format!("Opponent Time:  {:.3}s", opponent_et)),
        Line::from(format!(
            "Margin:         {:.3}s",
            (player_et - opponent_et).abs()
        )),
        Line::from(""),
        Line::from(format!("Reaction Time:  {:.3}s", player_rt)),
        Line::from(format!("Top Speed:      {:.1} m/s", race.player.top_speed)),
        Line::from(format!("Perfect Shifts: {}", race.player.perfect_shifts)),
        Line::from(""),
        Line::from(format!(
            "Audio: {}",
            if app.audio_muted {
                Span::styled("MUTED", Style::default().fg(Color::Red))
            } else {
                Span::styled("ON", Style::default().fg(Color::Green))
            }
        )),
        Line::from(""),
        Line::from("═══════════════════════════════"),
        Line::from(""),
        Line::from("[R] Race Again"),
        Line::from("[M] Toggle Audio"),
        Line::from("[Q] Quit to Menu"),
    ];

    let paragraph = Paragraph::new(results_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Race Results"));

    f.render_widget(paragraph, area);
}
