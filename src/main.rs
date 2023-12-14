use crate::iss::Iss;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
        KeyEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{MapResolution, Painter, Shape, Canvas, Map};
use chrono::prelude::*;
use ratatui::layout::Direction::{Horizontal, Vertical};
use std::{error::Error, io};
use OrbitalEphemerisMessage::Satellite;

pub mod iss;

// const ISS_DATA: [(f64, f64); 7] = [
//     (-3901.831067378710, -5313.183806503490),
//     (-4579.71120418877, -5028.721419956560),
//     (-4926.24313981341, -4379.407757890590),
//     (-4915.8195182088, -3411.881627490280),
//     (-4548.5594695655, -2196.010698574360),
//     (-3850.5423196244, -820.025684187015),
//     (-2872.12994324591, 615.821403372990),
// ];

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let url = OrbitalEphemerisMessage::ISS_OEM_URL;
    let content: Result<String, OrbitalEphemerisMessage::Error> = OrbitalEphemerisMessage::download_file(url);

    let mut sat = match content {
        Ok(content) => OrbitalEphemerisMessage::construct_oem(&content),
        Err(error) => {
            println!("Error downloading content: {}", error);
            // Return a default Satellite value if there was an error
            OrbitalEphemerisMessage::Satellite::default()
        }
    };


    let mut iss = Iss::new();
    iss.update_position();

    // startup: Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    // Initialize the terminal backend using crossterm
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;


    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, &mut iss, &mut sat);
    // Main application loop
    // loop {
    //     let utc: DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    //     let local: DateTime<Local> = Local::now();
        // Render the UI
        // terminal.draw(|f| {
        //     let size = f.size();
        //     let chunks = Layout::default()
        //         .direction(Direction::Horizontal)
        //         .constraints([
        //             Constraint::Ratio(3, 24),
        //             Constraint::Ratio(9, 24),
        //             Constraint::Ratio(12, 24),
        //         ])
        //         .split(size);
        //     let datasets2 = vec![Dataset::default()
        //         .name("pos")
        //         .marker(symbols::Marker::Dot)
        //         .style(Style::default().fg(Color::Yellow))
        //         .graph_type(GraphType::Line)
        //         .data(iss.pos_data.as_slice())];
        //     f.render_widget(map_canvas(&iss.lat, &iss.lon, &zoom),chunks[2]);
        //     // f.render_widget(Chart::new(datasets2.clone())
        //     //                     .block(           Block::default()
        //     //                                           .title("ISS Historical Position".cyan().bold())
        //     //                                           .borders(Borders::ALL),)
        //     //                     .x_axis(            Axis::default()
        //     //                                             .title("Lat")
        //     //                                             .style(Style::default().fg(Color::Gray))
        //     //                                             .bounds([-180.0, 180.0])
        //     //                                             .labels(vec!["-180".bold(), "0".into(), "180".bold()]),)
        //     //                     .y_axis(            Axis::default()
        //     //                                             .title("Lon")
        //     //                                             .style(Style::default().fg(Color::Gray))
        //     //                                             .bounds([-180.0, 180.0])
        //     //                                             .labels(vec!["-180".bold(), "0".into(), "180".bold()]),), chunks[1]);
        //     f.render_widget(Paragraph::new(format!("{0} \n{1}", sat.meta_summary, sat.trajectory_summary)).block(Block::default().borders(Borders::ALL).title("OEM DATA".cyan().bold())), chunks[1]);
        //     f.render_widget(Paragraph::new(format!("\n  Coordinates: \n LAT {0}  \n LON {1}  \n ALT {2} \n\n ISS Time: \n {3} \n Local Time: \n {4} \n\n Country: \n {5}", iss.lat, iss.lon, iss.alt, utc, local, iss.country)).block(Block::default().borders(Borders::ALL).title("ISS Tracker".cyan().bold())), chunks[0]);
        // })?;

    //     // Check for user input every 250 milliseconds
    //     if crossterm::event::poll(std::time::Duration::from_millis(250))? {
    //         // If a key event occurs, handle it
    //         if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
    //             if key.kind == crossterm::event::KeyEventKind::Press {
    //                 match key.code {
    //                     crossterm::event::KeyCode::Char('j') => counter += 1,
    //                     crossterm::event::KeyCode::Char('k') => counter -= 1,
    //                     crossterm::event::KeyCode::Char('u') => iss.update_position(),
    //                     crossterm::event::KeyCode::Char(']') => zoom-=10.0,
    //                     crossterm::event::KeyCode::Char('[') => zoom+=10.0,
    //                     crossterm::event::KeyCode::Char('q') => break,
    //                     _ => {},
    //                 }
    //             }
    //         }
    //     } else{
    //         duration += 250;
    //
    //         if duration >= 5500{
    //             iss.update_position();
    //             duration = 0;
    //         }
    //     }
    // }
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
    // // shutdown down: reset terminal back to original state
    // crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    // crossterm::terminal::disable_raw_mode()?;
    //
    // Ok(())

}

fn map_canvas(&lat: &f64,&lon: &f64, zoom: &f64) -> impl Widget + 'static {
    Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Current ISS Position".cyan().bold()))
        .marker(Marker::Braille)
        .paint(move |ctx| {
            ctx.draw(&Map {
                color: Color::Yellow,
                resolution: MapResolution::High,

            });
            ctx.print(lon, lat, "ISS".red().add_modifier(Modifier::BOLD)); //Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        })
        .x_bounds([lon-zoom, lon+zoom])
        .y_bounds([lat-zoom, lat+zoom])
}
pub fn ui(f: &mut Frame, app: &App,iss: &mut Iss,
          sat: &mut Satellite, zoom: f64) {

    let utc: DateTime<Utc> = Utc::now();       // e.g. `2014-11-28T12:45:59.324310806Z`
    let local: DateTime<Local> = Local::now();
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let mut content = "rsISS";

    let title = Paragraph::new(Text::styled(
        content,
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ))
        .block(title_block);

    f.render_widget(title, chunks[0]);

    let widget1 = Paragraph::new(format!("{0} \n{1}", sat.meta_summary, sat.trajectory_summary)).block(Block::default().borders(Borders::ALL).title("OEM DATA".cyan().bold()));
    let widget2 = Paragraph::new(format!("\n  Coordinates: \n LAT {0}  \n LON {1}  \n ALT {2} \n\n ISS Time: \n {3} \n Local Time: \n {4} \n\n Country: \n {5}", iss.lat, iss.lon, iss.alt, utc, local, iss.country)).block(Block::default().borders(Borders::ALL).title("ISS Tracker".cyan().bold()));
    let widget3 = map_canvas(&iss.lat, &iss.lon, &zoom);

    let current_widget= match app.current_screen {
        CurrentScreen::Main => {
            f.render_widget(widget1, chunks[1]);
        },
        CurrentScreen::Secondary => {
            f.render_widget(widget2, chunks[1])
        },

        CurrentScreen::Map => {
            f.render_widget(widget3, chunks[1])},

        _ => f.render_widget(widget1, chunks[1]),
    };


    if let CurrentScreen::Exiting = app.current_screen {
        f.render_widget(Clear, f.size()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to quit? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, f.size());
        f.render_widget(exit_paragraph, area);
    }


}

pub enum CurrentScreen {
    Main,
    Secondary,
    Map,
    Exiting,
}
// enum Internal_Widget {
//     Paragraph(Paragraph),
//     Map(map_canvas), // Replace Widget with the actual type returned by `map_canvas`
// }

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    iss: &mut Iss,
    sat: &mut Satellite,
) -> io::Result<bool> {
    let mut zoom = 50.0;
    let mut duration = 0;
    loop {
        terminal.draw(|f| ui(f, app, iss, sat, zoom))?;

        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match app.current_screen {
                    CurrentScreen::Main => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::Secondary;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        _ => {}
                    },
                    CurrentScreen::Secondary => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::Map;
                        }
                        KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Exiting;
                        }
                        KeyCode::Char('u') => {
                            iss.update_position();
                        }
                        _ => {}
                    },
                    CurrentScreen::Map => match key.code {
                        KeyCode::Char('l') => {
                            app.current_screen = CurrentScreen::Main;
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
                    CurrentScreen::Exiting => match key.code {
                        KeyCode::Char('y') => {
                            return Ok(true);
                        }
                        KeyCode::Char('n') | KeyCode::Char('q') => {
                            return Ok(false);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }else{
                    duration += 250;

                    if duration >= 5500{
                        iss.update_position();
                        duration = 0;
                    }
                }

    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}