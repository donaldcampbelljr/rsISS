use crate::iss::{get_position, Iss};
use std::{thread, time};


use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::Paragraph,
};

pub mod iss;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    //
    //
    let mut iss = Iss::new();
    // let delay = time::Duration::from_secs(2);
    // thread::sleep(delay);
    iss.update_position();
    //
    // println!("ENDING PROGRAM");

    // startup: Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    // Initialize the terminal backend using crossterm
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // Define our counter variable
    // This is the state of our application
    let mut counter = 0;

    // Main application loop
    loop {
        // Render the UI
        terminal.draw(|f| {
            f.render_widget(Paragraph::new(format!("Counter: {counter}\n\n ISS Position: {0}  {1}  {2}", iss.lat, iss.lon, iss.alt)), f.size());
        })?;

        // Check for user input every 250 milliseconds
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            // If a key event occurs, handle it
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('j') => counter += 1,
                        crossterm::event::KeyCode::Char('k') => counter -= 1,
                        crossterm::event::KeyCode::Char('u') => iss.update_position(),
                        crossterm::event::KeyCode::Char('q') => break,
                        _ => {},
                    }
                }
            }
        }
    }

    // shutdown down: reset terminal back to original state
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())



}
