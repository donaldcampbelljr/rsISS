use std::io::Read;
use serde_json::{Value};
#[derive(Debug,Default)]
pub struct Iss {
    pub lat: f64,
    pub lon: f64,
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
    }
}


pub fn get_position() -> Result<(f64,f64), Box<dyn std::error::Error>> {

    let mut res = reqwest::blocking::get("http://api.open-notify.org/iss-now.json")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;
    println!("Status: {}", res.status());
    println!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    //let json: Value = serde_json::from_str(&body)?;
    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    println!("The message  {} at the timestamp {}", json["message"], json["timestamp"]);
    println!("iss_position is {}", json["iss_position"]);
    println!("Latitude is {}", json["iss_position"]["latitude"]);
    println!("Longitude is {:?}", json["iss_position"]["longitude"]);
    //println!("Longitude is {:?}", json["iss_position"]["longitude"].as_number()); // THIS RETURNS None

    // for line below Take the JSON Value (which is actually a String()?), convert to &str (make sure to do expectation handling), parse the &str, doing expectation handling
    // pub fn as_str(&self) -> Option<&str>, If the Value is a String, returns the associated str. Returns None otherwise.
    let latitude: f64 = json["iss_position"]["latitude"]
        .as_str().expect("str expected") //https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html#method.as_str // returns str or None
        .parse().expect("Desire a number"); // parse needs to know what string it is expecting in this cae its an f64 // will return Err if it cannot parse.

    let longitude: f64 = json["iss_position"]["longitude"]
        .as_str().expect("str expected")
        .parse().expect("Desire a number");

    println!("Latitude is {:?}", latitude);
    println!("Longitude is {:?}", longitude);
    //Ok(())
    Ok((latitude,longitude))
}