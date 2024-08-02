use std::io::Read;
use std::str::FromStr;
use serde_json::{Value};
use rgeo::{search};
use rgeo::record::{Nvec, Record};
use country_emoji::{flag};
use std::string::String;
use serde_json::Value::String as JsonString;


pub const weather_ascii_sun: &str = 

"
   | \n
 ~ 0 ~\n
   |\n
";

pub const weather_ascii_clouds: &str = 

"
    ~~~~\n
   (░░░░) \n
 (░░░░░░)\n
 (░░░░░)\n
";

pub const weather_ascii_rain: &str = 

"
 !  !\n
  '   ' '! \n
 ' ' ! !\n
 !  !\n
";

pub const weather_ascii_wind: &str = 

"
 ~  ~ ~\n
   ~  ~  ~\n
 ~ ~ ~ ~\n
";

pub const weather_ascii_outer_space: &str = 

"
 *       *\n
      *   \n
   *    \n
";


#[derive(Debug,Default)]
pub struct Iss {
    pub lat: f64,
    pub lon: f64,
    pub alt: f64,
    pub time: f64,
    pub country: String,
    pub pos_data: Vec<(f64,f64)>,
    pub prev_alt: f64,
    pub alt_perigee_apogee: String,
    pub crew: String,
    pub weather: String,
}

impl Iss {
    /// Constructs a new instance of [`Iss`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_crew(&mut self)
    {
        let current_crew = get_crew().unwrap();
        self.crew = current_crew.join("\n");

    }

    pub fn update_weather(&mut self)
    {
        println!("Here is a PRINTED LINE");

        let weather = get_weather(self.lat, self.lon).unwrap();

        //println!("{:?}", weather);

        self.weather = weather;

    }

    /// Set running to false to quit the application.
    pub fn update_position(&mut self) {
        let new_position = get_position().unwrap();
        self.prev_alt = self.alt;
        self.lat = new_position.0;
        self.lon = new_position.1;
        self.alt = new_position.2;
        self.time = new_position.3;
        self.country = new_position.4;
        self.pos_data.push((new_position.0, new_position.1));
        if self.prev_alt > self.alt
        {
            self.alt_perigee_apogee = String::from("Approaching Perigee");
        }
        else
        {
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

    //let flag = flag(rgeo_result.1.country.as_str()).unwrap_or(String::from("Unknown Country"));
    //let countryString = String::from(rgeo_result.1.country.as_str()) + "\n" + flag.as_str();
    let countryString = String::from(rgeo_result.1.country.as_str());

    Ok(countryString)


}

pub fn get_crew() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut res = reqwest::blocking::get("http://api.open-notify.org/astros.json")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    //println!("{}", json["people"]);

    let new_array = json["people"].as_array().unwrap();

    let mut crew_member_list = Vec::new();
    for val in new_array.into_iter(){
        //println!("{}", val["name"]);
        crew_member_list.push(val["name"].to_string());
    }

    Ok((crew_member_list))
}

pub fn get_weather(lat: f64, lon: f64) ->  Result<String, Box<dyn std::error::Error>> {


    //let constructed_url = format!("https://api.weather.gov/points/{lat},{lon}").to_string();  //String::from_str("https://api.weather.gov/points/{lat},{lon}");

    //let constructed_url = format!("https://api.weather.gov/points/38.8894,-77.0352").to_string(); //basic example given on website

    //let constructed_url = format!("https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&current=temperature").to_string();

    let constructed_url = format!("https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature,weather_code").to_string();

    let mut res = reqwest::blocking::get(constructed_url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => return Err(Box::new(err)),
    };

    // let new_url = json["properties"]["forecast"].as_str().unwrap();

    // let mut res = reqwest::blocking::get(new_url)?;
    // let mut body = String::new();
    // res.read_to_string(&mut body)?;

    // let json: Value = match serde_json::from_str(&body) {
    //     Ok(json) => json,
    //     Err(err) => return Err(Box::new(err)),
    // };

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

pub fn get_weather_ascii(weather_code: &String) -> String {


    let code_int = match weather_code.parse::<i32>() {
        Ok(num) => num,
        Err(_) => 1001,
      };
    
      // Match the integer code to weather condition
      let condition = match code_int {
        0 => weather_ascii_sun,
        1..=3 => weather_ascii_sun,
        4..=8 => weather_ascii_clouds,
        51..=55 => weather_ascii_rain,
        56..=57 =>  weather_ascii_rain,
        61..=65 =>  weather_ascii_rain,
        66..=67 => weather_ascii_rain,
        71..=75 =>  weather_ascii_rain,
        77 =>  weather_ascii_rain,
        80..=82 =>  weather_ascii_rain,
        85..=86 =>  weather_ascii_rain,
        95 => weather_ascii_rain,
        96..=99 =>  weather_ascii_rain,
        _ =>  weather_ascii_outer_space,};

    let final_string = String::from_str(condition).unwrap();

    //let final_string = weather_ascii_sun.to_string();
    //let final_string = weather_ascii_wind.to_string();
    //let final_string = weather_ascii_rain.to_string();
    //let final_string = weather_ascii_outer_space.to_string();

    final_string

}

fn get_wmo_code(weather_code: &String) -> String{

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
        _ => "Unknown weather code".to_string(),};


        return condition


}