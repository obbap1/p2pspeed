use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct FastInfo {
    #[serde(rename(deserialize = "downloadSpeed"))]
    download_speed: i32,
    #[serde(rename(deserialize = "uploadSpeed"))]
    upload_speed: i32,
    downloaded: i32,
    uploaded: i32,
    latency: i32,
    #[serde(rename(deserialize = "bufferBloat"))]
    buffer_bloat: i32,
    #[serde(rename(deserialize = "userLocation"))]
    user_location: String,
    #[serde(rename(deserialize = "userIp"))]
    user_ip: String
}

pub fn fetch_info() -> FastInfo {
    let output = std::process::Command::new("fast")
                            .args(["--upload", "--json"]).output()
                            .expect("failed to execute fast process");
    let info = String::from_utf8_lossy(&output.stdout);
    let fast_info = serde_json::from_str(info.into_owned().as_str())
                        .expect("couldnt parse fast info");
    fast_info
}