//! Contains all the types to parse the messages that are send as messages on the websocket.

use geojson::GeoJson;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum WebSocket {
    Status { status: String },
    Pong(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheck {
    service: String,
    healthy: bool,
    tenant: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Properties {
    r#ref: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExtraGeoms {
    r#type: String,
    properties: Properties,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewsTickerMessage {
    title: String,
    lines: Vec<String>,
    content: String,
    updated: String, // TODO: use date time instead
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SbmNewsTicker {
    incident_program: Option<bool>,
    messages: Vec<NewsTickerMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "source", content = "content")]
pub enum Content {
    #[serde(rename = "trajectory_schematic")]
    TrajectorySchematic(GeoJson),
    #[serde(rename = "deleted_vehicles_schematic")]
    DeletedVehiclesSchematic(Option<String>),
    #[serde(rename = "station_schematic")]
    StationSchematic(GeoJson),
    #[serde(rename = "websocket")]
    Websocket(WebSocket),
    #[serde(rename = "extra_geoms")]
    ExtraGeoms(Option<ExtraGeoms>),
    #[serde(rename = "healthcheck")]
    Healthcheck(HealthCheck),
    #[serde(rename = "sbm_newsticker")]
    SbmNewsTicker(SbmNewsTicker),
    #[serde(rename = "trajectory")]
    Trajectory(GeoJson),
    #[serde(rename = "deleted_vehicles")]
    DeletedVehicles(Option<String>),
    #[serde(rename = "station")]
    Station(GeoJson),
}

// {"source": "deleted_vehicles_schematic", "content": "sbm_140404727073712", "timestamp": 1697454536271.5, "client_reference": null}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseMessage {
    // source: String,
    #[serde(flatten)]
    pub content: Content,
    pub timestamp: f64,
    client_reference: Option<i8>,
}
