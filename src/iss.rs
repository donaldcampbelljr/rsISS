use std::io::Read;
use serde_json::{Value};
#[derive(Debug,Default)]
pub struct Iss {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
}

impl Iss {
    /// Constructs a new instance of [`Iss`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set running to false to quit the application.
    pub fn update_position(&mut self) {
        let new_position = get_position().unwrap();
        self.lat = new_position.0;
        self.lon = new_position.1;
        self.alt = new_position.2;
    }
    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     write!(f, "ISS {{ latitude: {}, longitude: {} }}", self.lat, self.lon)
    // }

}


pub fn get_position() -> Result<(f64,f64, f64), Box<dyn std::error::Error>> {

    //let mut res = reqwest::blocking::get("http://api.open-notify.org/iss-now.json")?;
    // https://api.wheretheiss.at/v1/satellites/25544
    let mut res = reqwest::blocking::get("https://api.wheretheiss.at/v1/satellites/25544")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    let latitude: f64 = json["latitude"].as_f64().expect("Desire a number");
    let longitude: f64 = json["longitude"].as_f64().expect("Desire a number");
    let altitude: f64 = json["altitude"].as_f64().expect("Desire a number");
        // .as_str().expect("str expected")
        // .parse().expect("Desire a number");

    Ok((latitude,longitude, altitude))
}