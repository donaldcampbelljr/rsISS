#[cfg(not(target_arch = "wasm32"))]
use rgeo::record::{Nvec, Record};
#[cfg(not(target_arch = "wasm32"))]
use rgeo::search;

use serde_json::Value;
use std::io::Read;
use std::str::FromStr;
use std::string::String;
use serde::{Deserialize, Serialize};

pub const WEATHER_ASCII_SUN: &str = "
   | \n
 ~ 0 ~\n
   |\n
";

pub const WEATHER_ASCII_CLOUDS: &str = "
    ~~~~\n
   (░░░░) \n
 (░░░░░░)\n
 (░░░░░)\n
";

pub const WEATHER_ASCII_RAIN: &str = "
 !  !\n
  '   ' '! \n
 ' ' ! !\n
 !  !\n
";

pub const WEATHER_ASCII_WIND: &str = "
 ~  ~ ~\n
   ~  ~  ~\n
 ~ ~ ~ ~\n
";

pub const WEATHER_ASCII_OUTER_SPACE: &str = "
 *       *\n
      *   \n
   *    \n
";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iss {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub time: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub country: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub pos_data: Vec<(f64, f64)>,
    #[cfg(not(target_arch = "wasm32"))]
    pub prev_alt: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub alt_perigee_apogee: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub crew: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub weather: String,
}

impl Iss {
    /// Constructs a new instance of [`Iss`].
    pub fn new() -> Self {
        Iss {
            lat: 0.0,
            lon: 0.0,
            alt: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            time: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            pos_data: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            prev_alt: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            country: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            alt_perigee_apogee: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            crew: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            weather: String::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_crew(&mut self) {
        let current_crew = get_crew().unwrap();
        self.crew = current_crew.join("\n");
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_weather(&mut self) {
        let weather = get_weather(self.lat, self.lon).unwrap();

        self.weather = weather;
    }
        // WASM stubs (so the interface is consistent)
    #[cfg(target_arch = "wasm32")]
    pub fn update_crew(&mut self) {
        // No-op for WASM or simplified implementation
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update_weather(&mut self) {
        // No-op for WASM or simplified implementation
    }

    /// Set running to false to quit the application.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_position(&mut self) {
        let new_position = get_position().unwrap();
        self.prev_alt = self.alt;
        self.lat = new_position.0;
        self.lon = new_position.1;
        self.alt = new_position.2;
        self.time = new_position.3;
        self.country = new_position.4;
        self.pos_data.push((new_position.0, new_position.1));
        if self.prev_alt > self.alt {
            self.alt_perigee_apogee = String::from("Approaching Perigee");
        } else {
            self.alt_perigee_apogee = String::from("Approaching Apogee");
        }

        // these numbers are reported differently across the internet: 370-460 km as the altitude
        // 413 and 422 reported as the perigee and apogee
        if self.alt.floor() > 429.0 {
            self.alt_perigee_apogee = String::from("Apogee Reached");
        }

        if self.alt.floor() < 372.0 {
            self.alt_perigee_apogee = String::from("Perigee Reached");
        }
    }
#[cfg(target_arch = "wasm32")]
pub async fn update_position_async(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    let new_position = get_position_async().await?; // This can fail, so propagate the error
    self.lat = new_position.0;
    self.lon = new_position.1;
    self.alt = new_position.2;
    
    Ok(()) // Return success
}

}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_position() -> Result<(f64, f64, f64, f64, String), Box<dyn std::error::Error>> {
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
    let country: String = match get_country(latitude, longitude) {
        Ok(country) => country,
        Err(e) => "Unknown Country".to_string(),
    };

    Ok((latitude, longitude, altitude, timestamp, country))
}

#[cfg(target_arch = "wasm32")]
async fn get_position_async() -> Result<(f64, f64, f64, String), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://api.wheretheiss.at/v1/satellites/25544").await?;
    let json: Value = response.json().await?;
    
    let lat = json["latitude"].as_f64().unwrap();
    let lon = json["longitude"].as_f64().unwrap();
    let alt = json["altitude"].as_f64().unwrap();
    
    // No country lookup in WASM for now (rgeo doesn't work in WASM)
    let country = String::from("Unknown");
    
    Ok((lat, lon, alt, country))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_country(lat: f64, lon: f64) -> Result<String, Box<dyn std::error::Error>> {
    let latitude = lat;
    let longitude = lon;

    let default_record = Record {
        name: String::from("Unknown Location"),
        nvec: Nvec::from_lat_long(0.0, 0.0),
        country: String::from("Unknown Country"),
    };

    let rgeo_result = search(latitude as f32, longitude as f32).unwrap_or((0.0, &default_record));

    //let flag = flag(rgeo_result.1.country.as_str()).unwrap_or(String::from("Unknown Country"));
    //let countryString = String::from(rgeo_result.1.country.as_str()) + "\n" + flag.as_str();
    let countryString = String::from(rgeo_result.1.country.as_str());

    Ok(countryString)
}

#[cfg(target_arch = "wasm32")]
pub fn get_country(lat: f64, lon: f64) -> Result<String, Box<dyn std::error::Error>> {
    let latitude = lat;
    let longitude = lon;
    let countryString = String::from("The Undiscovered Country");

    Ok(countryString)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_crew() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut res = reqwest::blocking::get("http://api.open-notify.org/astros.json")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    let new_array = json["people"].as_array().unwrap();

    let mut crew_member_list = Vec::new();
    for val in new_array.into_iter() {
        crew_member_list.push(val["name"].to_string());
    }

    Ok((crew_member_list))
}
#[cfg(not(target_arch = "wasm32"))]
pub fn get_weather(lat: f64, lon: f64) -> Result<String, Box<dyn std::error::Error>> {
    let constructed_url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature,weather_code").to_string();

    let mut res = reqwest::blocking::get(constructed_url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    let mut forecast = json["current"]["temperature"].to_string();

    let weather_code = json["current"]["weather_code"].to_string();

    let wmo_forecast = get_wmo_code(&weather_code);

    forecast.push_str(" degrees");
    forecast.push_str("\n");
    forecast.push_str(&wmo_forecast);

    let ascii_weather = get_weather_ascii(&weather_code);

    forecast.push_str("\n");
    forecast.push_str(&ascii_weather);

    Ok(forecast)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_weather_ascii(weather_code: &String) -> String {
    let code_int = match weather_code.parse::<i32>() {
        Ok(num) => num,
        Err(_) => 1001,
    };

    // Match the integer code to weather condition
    let condition = match code_int {
        0 => WEATHER_ASCII_SUN,
        1..=3 => WEATHER_ASCII_SUN,
        4..=8 => WEATHER_ASCII_CLOUDS,
        51..=55 => WEATHER_ASCII_RAIN,
        56..=57 => WEATHER_ASCII_RAIN,
        61..=65 => WEATHER_ASCII_RAIN,
        66..=67 => WEATHER_ASCII_RAIN,
        71..=75 => WEATHER_ASCII_RAIN,
        77 => WEATHER_ASCII_RAIN,
        80..=82 => WEATHER_ASCII_RAIN,
        85..=86 => WEATHER_ASCII_RAIN,
        95 => WEATHER_ASCII_RAIN,
        96..=99 => WEATHER_ASCII_RAIN,
        _ => WEATHER_ASCII_OUTER_SPACE,
    };

    let final_string = String::from_str(condition).unwrap();

    final_string
}

#[cfg(not(target_arch = "wasm32"))]
fn get_wmo_code(weather_code: &String) -> String {
    let code_int = match weather_code.parse::<i32>() {
        Ok(num) => num,
        Err(_) => 1001,
    };

    // Match the integer code to weather condition
    let condition = match code_int {
        0 => "Clear sky".to_string(),
        1..=3 => "Mainly clear, partly cloudy, or overcast".to_string(),
        4..=8 => "Fog".to_string(),
        51..=55 => "Drizzle (Light, moderate, or dense)".to_string(),
        56..=57 => "Freezing Drizzle (Light or dense)".to_string(),
        61..=65 => "Rain (Slight, moderate, or heavy)".to_string(),
        66..=67 => "Freezing Rain (Light or heavy)".to_string(),
        71..=75 => "Snowfall (Slight, moderate, or heavy)".to_string(),
        77 => "Snow grains".to_string(),
        80..=82 => "Rain showers (Slight, moderate, or violent)".to_string(),
        85..=86 => "Snow showers (Slight or heavy)".to_string(),
        95 => "Thunderstorm (Slight or moderate)".to_string(),
        96..=99 => "Thunderstorm with slight or heavy hail".to_string(),
        _ => "Unknown weather code".to_string(),
    };

    return condition;
}
