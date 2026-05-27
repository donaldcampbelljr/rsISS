use crate::app::{App, CurrentScreen};
use crate::iss::Iss;
use chrono::prelude::*;
use chrono::Duration;
use ratatui::widgets::canvas::{Canvas, Map, MapResolution};
use ratatui::{prelude::*, widgets::*};
use crate::oem::Satellite;

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
) {
    let utc: DateTime<Utc> = Utc::now();
    let local: DateTime<Local> = Local::now();

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

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let title = Paragraph::new(Text::styled(
        "rsISS",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(title_block);

    f.render_widget(title, chunks[0]);

    let footer = Paragraph::new(Text::styled(
        format!("CURRENT RUN TIME: {0}", elapsed_time),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
    .block(footer_block);

    f.render_widget(footer, footer_inner_layout[0]);

    let footer_instructions = Paragraph::new(Text::styled(
        "VIEWS: 'l' UPDATE: 'u' ZOOM: '[' ']'  QUIT: 'q'",
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

    let tracking_widget = Paragraph::new(format!(
        "\n Coordinates: \n LAT {0}  \n LON {1}  \n ALT {2} \n\n ISS Time: \n {3} \n Local Time: \n {4} \n\n Country: \n {5} \n\n Additional Info: \n {6}",
        iss.lat, iss.lon, iss.alt, utc, local, iss.country, iss.alt_perigee_apogee
    ))
    .block(Block::default().borders(Borders::ALL).title("ISS Tracker".cyan().bold()));

    let map_widget = map_canvas(&iss.lat, &iss.lon, &zoom);

    let trajectory_widget = Paragraph::new(format!("{0}", sat.trajectory_summary))
        .block(Block::default().borders(Borders::ALL).title("Upcoming".cyan().bold()));

    let coordinates_widget = Paragraph::new(format!("{0}", sat.coordinates))
        .block(Block::default().borders(Borders::ALL).title("Future Trajectories".cyan().bold()));

    let crew_widget = Paragraph::new(format!("{0}", iss.crew))
        .block(Block::default().borders(Borders::ALL).title("Current ISS Crew".cyan().bold()));

    let weather_widget = Paragraph::new(format!("{0}", iss.weather))
        .block(Block::default().borders(Borders::ALL).title("Current Weather at Coordinates".cyan().bold()));

    match app.current_screen {
        CurrentScreen::Tracker => {
            f.render_widget(tracking_widget, inner_layout[0]);
            f.render_widget(map_widget, inner_layout[1]);
        }
        CurrentScreen::FullMap => {
            f.render_widget(map_widget, chunks[1]);
        }
        CurrentScreen::UpcomingEvents => {
            f.render_widget(trajectory_widget, inner_layout2[0]);
            f.render_widget(coordinates_widget, inner_layout2[1]);
        }
        CurrentScreen::Crew => {
            f.render_widget(crew_widget, inner_layout2[0]);
            f.render_widget(weather_widget, inner_layout2[1]);
        }
        _ => f.render_widget(tracking_widget, chunks[1]),
    }

    if let CurrentScreen::Exiting = app.current_screen {
        f.render_widget(Clear, f.size());
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to quit? (y/n)",
            Style::default().fg(Color::Red),
        );
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, f.size());
        f.render_widget(exit_paragraph, area);
    }
}

/// Helper function to create a centered rect using up a certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
