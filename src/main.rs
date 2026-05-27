#![cfg(not(target_arch = "wasm32"))]

mod app;
pub mod iss;
mod ui;

use app::{App, CurrentScreen};
use crate::iss::Iss;
use chrono::prelude::*;
use chrono::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use ui::ui;
use OrbitalEphemerisMessage::Satellite;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nLoading Orbital Data....");

    let start_time: DateTime<Local> = Local::now();

    let url = OrbitalEphemerisMessage::ISS_OEM_URL;
    let content = OrbitalEphemerisMessage::download_file(url);

    let mut sat = match content {
        Ok(content) => OrbitalEphemerisMessage::construct_oem(&content),
        Err(error) => {
            println!("Error downloading content: {}", error);
            OrbitalEphemerisMessage::Satellite::default()
        }
    };

    let mut iss = Iss::new();
    iss.alt = 417.5;
    iss.update_crew();
    iss.update_position();
    iss.update_weather();

    // Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::new();
    run_app(&mut terminal, &mut app, &mut iss, &mut sat, start_time)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    iss: &mut Iss,
    sat: &mut Satellite,
    start_time: DateTime<Local>,
) -> io::Result<bool> {
    let mut zoom = 50.0;
    let mut duration = 0;
    loop {
        let elapsed_time: Duration = Local::now() - start_time;

        terminal.draw(|f| ui(f, app, iss, sat, zoom, elapsed_time))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match app.current_screen {
                    CurrentScreen::Tracker => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::FullMap;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        KeyCode::Char(']') => {
                            zoom -= 10.0;
                        }
                        KeyCode::Char('[') => {
                            zoom += 10.0;
                        }
                        _ => {}
                    },
                    CurrentScreen::FullMap => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::UpcomingEvents;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        KeyCode::Char(']') => {
                            zoom -= 10.0;
                        }
                        KeyCode::Char('[') => {
                            zoom += 10.0;
                        }
                        _ => {}
                    },
                    CurrentScreen::UpcomingEvents => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::Crew;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        _ => {}
                    },
                    CurrentScreen::Crew => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::Tracker;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        _ => {}
                    },
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            return Ok(true);
                        }
                        KeyCode::Char('n') | KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Tracker;
                        }
                        _ => {}
                    },
                }
            }
        } else {
            duration += 250;

            if duration >= 5500 {
                iss.update_position();
                duration = 0;
            }
        }
    }
}
