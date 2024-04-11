use chrono::Utc;
use reqwest::Client;
use reqwest::Response; 
use dotenv::dotenv;
use serde::Serialize;
use serde_json::Value;
use std::env::var;
use std::env::VarError;
use chrono::TimeZone;


#[tokio::main]
async fn main() {
    dotenv().ok();
    let position = match Position::read_from_env() {
        Ok(data) => data,
        Err(e) => panic!("Error reading from env: {}", e),
    };

    let response = match Api::get_current_weather(&position).await{
        Ok(resp) => resp,
        Err(e) => panic!("Error fetching weather: {}", e),
    };

    let json = match Api::handle_response(response).await{
        Ok(json) => json,
        Err(status_code) => panic!("Error: {}", status_code),
    };

    let weather_data = WeatherData::from_json(&json, position);
    
    match Api::send_request(weather_data).await {
        Ok(_) => println!("Data sent successfully"),
        Err(e) => panic!("Error sending data: {}", e),
    };
}


impl Position{
    fn read_from_env() -> Result<Self, VarError>{
        Ok(Self{
            city: var("CITY")?,
            state: var("STATE")?,
            country: var("COUNTRY")?,
        })
    }
}


impl WeatherData{
    pub fn from_json(json: &serde_json::Value, position: Position) -> Self{
        let date = &json["data"]["current"]["weather"]["ts"].as_str().unwrap();
        //let timestamp = chrono::NaiveDateTime::from_timestamp(timestamp_seconds, 0).to_string();
        let datetime = Utc.datetime_from_str(date, "%Y-%m-%dT%H:%M:%S%.3fZ").expect("Failed to parse the date");
        let timestamp = datetime.timestamp();


        Self{
            timestamp,
            temperature: json["data"]["current"]["weather"]["tp"].as_f64().unwrap(),
            pressure: json["data"]["current"]["weather"]["pr"].as_f64().unwrap(),
            wind_speed: json["data"]["current"]["weather"]["ws"].as_f64().unwrap(),
            position
        }
    }
}


impl Api{
    async fn send_request(payload: WeatherData) -> Result<Response, reqwest::Error> {
        let url = "http://localhost:5000/add";
        let resp = Client::new().post(url).json(&payload).send().await;
        println!("resp {:#?}", resp);
        resp
    }


    async fn get_current_weather(data: &Position) -> Result<Response, reqwest::Error> {
        let Position {city, state, country} = data;
        let url = format!("http://api.airvisual.com/v2/city?city={city}&state={state}&country={country}&key=fb2b6665-61a2-432e-9582-99cecfc91f97");
        reqwest::get(&url).await
    }


    async fn handle_response(response: Response) -> Result<Value, u16>{
        let status_code = response.status().as_u16();
    
        match status_code {
            200 => (),
            429 => {println!("Rate limit exceeded, try again later"); return Err(429)},
            _ => return Err(status_code),
        } 
    
        match response.json::<serde_json::Value>().await {
            Ok(json) => Ok(json),
            Err(_) => Err(500),
        }
    }
}


struct Api;


#[derive(Debug, serde::Deserialize, Serialize)]
struct Position{
    city: String,
    state: String,
    country: String,
}


#[derive(Debug, Serialize)]
struct WeatherData{
    pub timestamp: i64,
    pub temperature: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub position: Position,

}