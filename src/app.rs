pub enum CurrentScreen {
    Tracker,
    FullMap,
    UpcomingEvents,
    Crew,
    Exiting,
}

pub struct App {
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Tracker,
        }
    }
}
