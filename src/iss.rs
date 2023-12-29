use std::io::Read;
use serde_json::{Value};
use rgeo::{search};
use rgeo::record::{Nvec, Record};
use country_emoji::{flag};
#[derive(Debug,Default)]
pub struct Iss {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub time: f64,
    pub country: String,
    pub pos_data: Vec<(f64,f64)>,
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
        self.time = new_position.3;
        self.country = new_position.4;
        self.pos_data.push((new_position.0, new_position.1));
    }
    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     write!(f, "ISS {{ latitude: {}, longitude: {} }}", self.lat, self.lon)
    // }

}


pub fn get_position() -> Result<(f64,f64, f64, f64, String), Box<dyn std::error::Error>> {

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
    let timestamp: f64 = json["timestamp"].as_f64().expect("Desire a number");
    let country: String = match get_country(latitude, longitude){
        Ok(country) => country,
        Err(e) => "Unknown Country".to_string(),
    };

    Ok((latitude,longitude, altitude, timestamp, country))
}

pub fn get_country(lat: f64, lon: f64) -> Result<String,Box<dyn std::error::Error>>{

    let latitude = lat;
    let longitude =lon;

    let default_record = Record{
        name: String::from("Unknown Location"),
        nvec: Nvec::from_lat_long(0.0, 0.0),
        country: String::from("Unknown Country"),
    };

    let rgeo_result =  search(latitude as f32, longitude as f32).unwrap_or((0.0, &default_record));

    let flag = flag(rgeo_result.1.country.as_str()).unwrap_or(String::from("Unknown Country"));

    let countryString = String::from(rgeo_result.1.country.as_str()) + "\n" + flag.as_str();

    Ok(countryString)


}