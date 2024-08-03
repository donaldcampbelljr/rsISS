use crate::iss::Iss;
use chrono::prelude::*;
use chrono::Duration;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use ratatui::{prelude::*, widgets::*};
use std::io;
use OrbitalEphemerisMessage::Satellite;

pub mod iss;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nLoading Orbital Data....");

    let start_time: DateTime<Local> = Local::now();

    let url = OrbitalEphemerisMessage::ISS_OEM_URL;
    let content: Result<String, OrbitalEphemerisMessage::Error> =
        OrbitalEphemerisMessage::download_file(url);

    let mut sat = match content {
        Ok(content) => OrbitalEphemerisMessage::construct_oem(&content),
        Err(error) => {
            println!("Error downloading content: {}", error);
            // Return a default Satellite value if there was an error
            OrbitalEphemerisMessage::Satellite::default()
        }
    };

    // Get min, max and then zip so that they can be plotted appropriately.
    let min_x = sat
        .x_coord_vec
        .iter()
        .fold(f64::INFINITY, |acc, &num| acc.min(num));
    let max_x = sat
        .x_coord_vec
        .iter()
        .fold(f64::NEG_INFINITY, |acc, &num| acc.max(num));

    let min_y = sat
        .y_coord_vec
        .iter()
        .fold(f64::INFINITY, |acc, &num| acc.min(num));
    let max_y = sat
        .y_coord_vec
        .iter()
        .fold(f64::NEG_INFINITY, |acc, &num| acc.max(num));

    let zipped_coords = sat.x_coord_vec.iter().zip(sat.y_coord_vec.iter());
    let future_coords: Vec<(f64, f64)> = zipped_coords.map(|(&x, &y)| (x, y)).collect();

    let mut iss = Iss::new();
    iss.alt = 417.5;
    iss.update_crew();
    iss.update_position();
    iss.update_weather();

    // startup: Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    // Initialize the terminal backend using crossterm
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut app = App::new();
    let res = run_app(
        &mut terminal,
        &mut app,
        &mut iss,
        &mut sat,
        start_time,
        future_coords,
        min_x,
        max_x,
        min_y,
        max_y,
    );

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn map_canvas(&lat: &f64, &lon: &f64, zoom: &f64) -> impl Widget + 'static {
    Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Current ISS Position".cyan().bold()),
        )
        .marker(Marker::Braille)
        .paint(move |ctx| {
            ctx.draw(&Map {
                color: Color::Yellow,
                resolution: MapResolution::High,
            });
            ctx.print(lon, lat, "ISS".red().add_modifier(Modifier::BOLD));
        })
        .x_bounds([lon - zoom, lon + zoom])
        .y_bounds([lat - zoom, lat + zoom])
}
pub fn ui(
    f: &mut Frame,
    app: &App,
    iss: &mut Iss,
    sat: &mut Satellite,
    zoom: f64,
    elapsed_time: Duration,
    future_coords: Vec<(f64, f64)>,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) {
    let utc: DateTime<Utc> = Utc::now(); // e.g. `2014-11-28T12:45:59.324310806Z`
    let local: DateTime<Local> = Local::now();
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    let inner_layout2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    let footer_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let title_inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[0]);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let title_content = "rsISS";

    let title = Paragraph::new(Text::styled(
        title_content,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(title_block);

    f.render_widget(title, chunks[0]);

    let footer_content = format!("CURRENT RUN TIME: {0}", elapsed_time);

    let footer_instructions_content = format!("VIEWS: 'l' UPDATE: 'u' ZOOM: '[' ']'  QUIT: 'q'");

    let footer = Paragraph::new(Text::styled(
        footer_content,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(footer_block);

    f.render_widget(footer, footer_inner_layout[0]);

    let footer_instructions = Paragraph::new(Text::styled(
        footer_instructions_content,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray)),
    );

    f.render_widget(footer_instructions, footer_inner_layout[1]);

    let tracking_widget = Paragraph::new(format!("\n Coordinates: \n LAT {0}  \n LON {1}  \n ALT {2} \n\n ISS Time: \n {3} \n Local Time: \n {4} \n\n Country: \n {5} \n\n Additional Info: \n {6}", iss.lat, iss.lon, iss.alt, utc, local, iss.country, iss.alt_perigee_apogee)).block(Block::default().borders(Borders::ALL).title("ISS Tracker".cyan().bold()));
    let map_widget = map_canvas(&iss.lat, &iss.lon, &zoom);
    let trajectory_widget = Paragraph::new(format!("{0}", sat.trajectory_summary)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Upcoming".cyan().bold()),
    );
    let coordinates_widget = Paragraph::new(format!("{0}", sat.coordinates)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Future Trajectories".cyan().bold()),
    );

    let future_coords_datasets = vec![Dataset::default()
        .name("pos")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(future_coords.as_slice())];

    let minx_string = format!("{}", min_x);
    let maxx_string = format!("{}", max_x);
    let miny_string = format!("{}", min_y);
    let maxy_string = format!("{}", max_y);

    // let future_coordinates_widget = Chart::new(future_coords_datasets.clone())
    //     .block(           Block::default()
    //                           .title("ISS Future Coordinates".cyan().bold())
    //                           .borders(Borders::ALL),)
    //     .x_axis(            Axis::default()
    //                             .title("X Coordinate (km)")
    //                             .style(Style::default().fg(Color::Gray))
    //                             .bounds([min_x, max_x])
    //                             .labels(vec![minx_string.bold(), "0".into(), maxx_string.bold()]),)
    //     .y_axis(            Axis::default()
    //                             .title("Y Coordinate (km)")
    //                             .style(Style::default().fg(Color::Gray))
    //                             .bounds([min_y, max_y])
    //                             .labels(vec![miny_string.bold(), "0".into(), maxy_string.bold()]));

    //     f.render_widget(map_canvas(&iss.lat, &iss.lon, &zoom),chunks[2]);
    //     f.render_widget(Chart::new(datasets2.clone())
    //                         .block(           Block::default()
    //                                               .title("ISS Historical Position".cyan().bold())
    //                                               .borders(Borders::ALL),)
    //                         .x_axis(            Axis::default()
    //                                                 .title("Lat")
    //                                                 .style(Style::default().fg(Color::Gray))
    //                                                 .bounds([-180.0, 180.0])
    //                                                 .labels(vec!["-180".bold(), "0".into(), "180".bold()]),)
    //                         .y_axis(            Axis::default()
    //                                                 .title("Lon")
    //                                                 .style(Style::default().fg(Color::Gray))
    //                                                 .bounds([-180.0, 180.0])
    //                                                 .labels(vec!["-180".bold(), "0".into(), "180".bold()]),), chunks[1]);

    let crew_widget = Paragraph::new(format!("{0}", iss.crew)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current ISS Crew".cyan().bold()),
    );

    let weather_widget = Paragraph::new(format!("{0}", iss.weather)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current Weather at Coordinates".cyan().bold()),
    );

    let current_widget = match app.current_screen {
        CurrentScreen::Tracker => {
            f.render_widget(tracking_widget, inner_layout[0]);
            f.render_widget(map_widget, inner_layout[1])
        }
        CurrentScreen::FullMap => {
            f.render_widget(map_widget, chunks[1]);
        }

        CurrentScreen::UpcomingEvents => {
            f.render_widget(trajectory_widget, inner_layout2[0]);
            f.render_widget(coordinates_widget, inner_layout2[1])
        }
        // CurrentScreen::Charts => {
        //     f.render_widget(future_coordinates_widget, inner_layout2[0]);
        //     f.render_widget(coordinates_widget, inner_layout2[1])
        // },
        CurrentScreen::Crew => {
            f.render_widget(crew_widget, inner_layout2[0]);
            f.render_widget(weather_widget, inner_layout2[1])
        }

        _ => f.render_widget(tracking_widget, chunks[1]),
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
    Tracker,
    FullMap,
    UpcomingEvents,
    Crew,
    Exiting,
}

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Tracker,
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    iss: &mut Iss,
    sat: &mut Satellite,
    start_time: DateTime<Local>,
    future_coords: Vec<(f64, f64)>,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> io::Result<bool> {
    let mut zoom = 50.0;
    let mut duration = 0;
    loop {
        let elapsed_time: Duration = Local::now() - start_time;

        terminal.draw(|f| {
            ui(
                f,
                app,
                iss,
                sat,
                zoom,
                elapsed_time,
                future_coords.clone(),
                min_x,
                max_x,
                min_y,
                max_y,
            )
        })?;

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

                    // CurrentScreen::Charts => match key.code {
                    //     KeyCode::Char('l') => {
                    //         app.current_screen = CurrentScreen::Crew;
                    //     }
                    //     KeyCode::Char('q') => {
                    //         app.current_screen = CurrentScreen::Exiting;
                    //     }
                    //     KeyCode::Char('u') => {
                    //         iss.update_position();
                    //     }
                    //     _ => {}
                    // },
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
                            return Ok(false);
                        }
                        _ => {}
                    },
                    _ => {}
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
