use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use serde::Serialize;
use crate::iss::Iss;

#[derive(Serialize)]
pub struct IssPosition {
    lat: f64,
    lon: f64,
    alt: f64,
    timestamp: String,
}

#[wasm_bindgen]
pub struct IssTracker {
    iss: Iss,
}

#[wasm_bindgen]
impl IssTracker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> IssTracker {
        console_error_panic_hook::set_once();
        
        let mut iss = Iss::new();
        iss.alt = 417.5;
        
        IssTracker { iss }
    }

    #[wasm_bindgen]
    pub fn get_position(&self) -> JsValue {
        let position = IssPosition {
            lat: self.iss.lat,
            lon: self.iss.lon,
            alt: self.iss.alt,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        serde_wasm_bindgen::to_value(&position).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_coordinates_string(&self) -> String {
        format!("LAT: {:.4}, LON: {:.4}, ALT: {:.1} km", 
                self.iss.lat, self.iss.lon, self.iss.alt)
    }

    // Async version that fetches real data
    #[wasm_bindgen]
    pub fn update_position_async(&mut self) -> js_sys::Promise {
        let mut iss_clone = self.iss.clone();
        
        future_to_promise(async move {
            match iss_clone.update_position_async().await {
                Ok(_) => Ok(JsValue::from("Success")),
                Err(e) => Err(JsValue::from_str(&format!("Error: {}", e)))
            }
        })
    }

    // Method to update the internal state after async operation
    #[wasm_bindgen]
    pub fn set_position(&mut self, lat: f64, lon: f64, alt: f64) {
        self.iss.lat = lat;
        self.iss.lon = lon;
        self.iss.alt = alt;
    }
}