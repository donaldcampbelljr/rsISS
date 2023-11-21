use crate::iss::{get_position, Iss};
use std::{thread, time};


// use ratatui::{
//     prelude::{CrosstermBackend, Terminal},
//     widgets::Paragraph,
// };

use ratatui::{prelude::*, widgets::*};

pub mod iss;

const ISS_DATA: [(f64, f64); 7] = [
    (-3901.831067378710, -5313.183806503490),
    (-4579.71120418877, -5028.721419956560),
    (-4926.24313981341, -4379.407757890590),
    (-4915.8195182088, -3411.881627490280),
    (-4548.5594695655, -2196.010698574360),
    (-3850.5423196244, -820.025684187015),
    (-2872.12994324591, 615.821403372990),
];


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

    let datasets = vec![Dataset::default()
        .name("data")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&ISS_DATA)];
    // let chart = Chart::new(datasets)
    //     .block(
    //         Block::default()
    //             .title("Chart 3".cyan().bold())
    //             .borders(Borders::ALL),
    //     )
    //     .x_axis(
    //         Axis::default()
    //             .title("X Axis")
    //             .style(Style::default().fg(Color::Gray))
    //             .bounds([0.0, 50.0])
    //             .labels(vec!["0".bold(), "25".into(), "50".bold()]),
    //     )
    //     .y_axis(
    //         Axis::default()
    //             .title("Y Axis")
    //             .style(Style::default().fg(Color::Gray))
    //             .bounds([0.0, 5.0])
    //             .labels(vec!["0".bold(), "2.5".into(), "5".bold()]),
    //     );

    // Define our counter variable
    // This is the state of our application
    let mut counter = 0;

    // Main application loop
    loop {
        // Render the UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ])
                .split(size);
            f.render_widget(Chart::new(datasets.clone())
                                .block(           Block::default()
                                                      .title("ISS Historical Position".cyan().bold())
                                                      .borders(Borders::ALL),)
                                .x_axis(            Axis::default()
                                                        .title("X Axis")
                                                        .style(Style::default().fg(Color::Gray))
                                                        .bounds([-4000.0, -3000.0])
                                                        .labels(vec!["0".bold(), "25".into(), "50".bold()]),)
                                .y_axis(            Axis::default()
                                                        .title("Y Axis")
                                                        .style(Style::default().fg(Color::Gray))
                                                        .bounds([-5000.0, 500.0])
                                                        .labels(vec!["0".bold(), "2.5".into(), "5".bold()]),), chunks[0]);
            f.render_widget(Paragraph::new(format!("Counter: {counter}\n\n ISS Position: {0}  {1}  {2}", iss.lat, iss.lon, iss.alt)), chunks[1]);
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
