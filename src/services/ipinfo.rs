use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug, Default, Deserialize)]
pub struct IPInfo {
    status: String,
    country: String,
    #[serde(rename(deserialize = "countryCode"))]
    country_code: String,
    region: String,
    #[serde(rename(deserialize = "regionName"))]
    region_name: String,
    city: String,
    zip: String,
    lat: f64,
    lon: f64,
    timezone: String,
    isp: String,
    org: String,
    r#as: String,
    query: String
}

pub async fn fetch_info() -> IPInfo {
    let response = reqwest::get("http://ip-api.com/json")
                                .await.expect("IP API is healthy")
                                .text().await.expect("Valid body content");
    let ip_info: IPInfo = serde_json::from_str(&response).expect("Valid json");
    ip_info
}