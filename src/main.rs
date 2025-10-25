use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

mod app;
mod game;
mod ui;

use app::{App, AppState};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16); // ~60 FPS

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
            app.update(last_tick.elapsed().as_secs_f64());
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
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
            (KeyCode::Char('q'), KeyEventKind::Press) => return true, // Quit app
            (KeyCode::Esc, KeyEventKind::Press) => {
                app.reset_all_key_states();
                app.state = AppState::Menu;
            },
            _ => {} // Ignore all other events including key releases
        },
    }
    false
}
