use crate::iss::Iss;

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{MapResolution, Painter, Shape, Canvas, Map};

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
    let mut iss = Iss::new();
    iss.update_position();

    // startup: Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    // Initialize the terminal backend using crossterm
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let mut counter = 0;
    let mut zoom = 50.0;
    let mut duration = 0;

    // Main application loop
    loop {
        // Render the UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Ratio(1, 9),
                    Constraint::Ratio(3, 9),
                    Constraint::Ratio(5, 9),
                ])
                .split(size);
            let datasets2 = vec![Dataset::default()
                .name("pos")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(iss.pos_data.as_slice())];
            f.render_widget(map_canvas(&iss.lat, &iss.lon, &zoom),chunks[2]);
            f.render_widget(Chart::new(datasets2.clone())
                                .block(           Block::default()
                                                      .title("ISS Historical Position".cyan().bold())
                                                      .borders(Borders::ALL),)
                                .x_axis(            Axis::default()
                                                        .title("Lat")
                                                        .style(Style::default().fg(Color::Gray))
                                                        .bounds([-180.0, 180.0])
                                                        .labels(vec!["-180".bold(), "0".into(), "180".bold()]),)
                                .y_axis(            Axis::default()
                                                        .title("Lon")
                                                        .style(Style::default().fg(Color::Gray))
                                                        .bounds([-180.0, 180.0])
                                                        .labels(vec!["-180".bold(), "0".into(), "180".bold()]),), chunks[1]);
            f.render_widget(Paragraph::new(format!("Counter: {counter}\n\n ISS Coordinates: \n LAT {0}  \n LON {1}  \n ALT {2}", iss.lat, iss.lon, iss.alt)), chunks[0]);
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
                        crossterm::event::KeyCode::Char(']') => zoom-=10.0,
                        crossterm::event::KeyCode::Char('[') => zoom+=10.0,
                        crossterm::event::KeyCode::Char('q') => break,
                        _ => {},
                    }
                }
            }
        } else{
            duration += 250;

            if duration >= 5500{
                iss.update_position();
                duration = 0;
            }
        }
    }

    // shutdown down: reset terminal back to original state
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())

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
            ctx.print(lat, lon, "ISS".red());
        })
        .x_bounds([lat-zoom, lat+zoom])
        .y_bounds([lon-zoom, lon+zoom])
}
