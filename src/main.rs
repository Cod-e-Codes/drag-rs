use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

mod app;
mod audio;
mod game;
mod ui;

use app::{App, AppState};
use audio::{AudioEngine, BeepType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize audio engine
    let audio = match AudioEngine::new() {
        Ok(engine) => Some(engine),
        Err(_e) => None, // Silently fail if audio can't be initialized
    };

    // Create app
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16); // ~60 FPS
    let mut last_light_state = game::LightState::PreStage;

    // Run app
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && handle_input(&mut app, key)
        {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            let delta = last_tick.elapsed().as_secs_f64();
            app.update(delta);

            // Update audio
            if let Some(ref audio_engine) = audio {
                // Update beep timer
                audio_engine.update_beeps(delta as f32);

                match app.state {
                    AppState::Racing => {
                        if let Some(race) = &app.race_state {
                            // Play beeps for Christmas tree state changes
                            if race.christmas_tree.state != last_light_state {
                                match race.christmas_tree.state {
                                    game::LightState::Yellow1
                                    | game::LightState::Yellow2
                                    | game::LightState::Yellow3 => {
                                        audio_engine.play_beep(BeepType::Yellow);
                                    }
                                    game::LightState::Green => {
                                        audio_engine.play_beep(BeepType::Green);
                                    }
                                    game::LightState::Racing => {
                                        // Check for red light
                                        if let Some(rt) = race.player.reaction_time
                                            && rt < 0.0
                                        {
                                            audio_engine.play_beep(BeepType::RedLight);
                                        }
                                    }
                                    _ => {}
                                }
                                last_light_state = race.christmas_tree.state;
                            }

                            // Update engine sound based on player RPM
                            audio_engine.update_engine(
                                race.player.rpm,
                                race.player.throttle as f32,
                                app.player_car.redline,
                            );
                        }
                    }
                    AppState::Menu | AppState::Results => {
                        // Silence in menu and results
                        audio_engine.stop();
                        last_light_state = game::LightState::PreStage;
                    }
                }
            }

            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Stop audio before exiting
    if let Some(audio_engine) = audio {
        audio_engine.stop();
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    match app.state {
        AppState::Menu => match (key.code, key.kind) {
            (KeyCode::Char('q') | KeyCode::Esc, KeyEventKind::Press) => return true,
            (KeyCode::Char('1'), KeyEventKind::Press) => app.start_race(),
            (KeyCode::Left, KeyEventKind::Press) => app.select_previous_car(),
            (KeyCode::Right, KeyEventKind::Press) => app.select_next_car(),
            _ => {}
        },
        AppState::Racing => match (key.code, key.kind) {
            (KeyCode::Char(' '), KeyEventKind::Press) => app.set_throttle_pressed(true),
            (KeyCode::Char(' '), KeyEventKind::Release) => app.set_throttle_pressed(false),
            (KeyCode::Up | KeyCode::Char('w'), KeyEventKind::Press) => app.shift_up(),
            (KeyCode::Up | KeyCode::Char('w'), KeyEventKind::Release) => app.reset_shift_state(),
            (KeyCode::Char('n'), KeyEventKind::Press) => app.set_nitrous_pressed(true),
            (KeyCode::Char('n'), KeyEventKind::Release) => app.set_nitrous_pressed(false),
            (KeyCode::Esc, KeyEventKind::Press) => {
                app.reset_all_key_states();
                app.state = AppState::Menu;
            }
            _ => {}
        },
        AppState::Results => match (key.code, key.kind) {
            (KeyCode::Char('r'), KeyEventKind::Press) => app.start_race(),
            (KeyCode::Char('q'), KeyEventKind::Press) => return true,
            (KeyCode::Esc, KeyEventKind::Press) => {
                app.reset_all_key_states();
                app.state = AppState::Menu;
            }
            _ => {}
        },
    }
    false
}
