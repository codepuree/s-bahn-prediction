#![feature(type_name_of_val)]

use geojson::GeoJson;
use serde_json::Value;
use serde_json::{self, Map};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::ops::{Deref, DerefMut};
use std::string::String;

use std::any::type_name_of_val;
use std::time::Duration;
use std::{thread, usize};

use macroquad::prelude::*;

use scraper::response_messages::{Content, ResponseMessage};

#[derive(Debug)]
enum AnalysisError {
    MissingProperty(String),
    IncorrectType(String, String),
    IncorrectValueType(String, String, String),
    MissingItems(usize, usize),
}

impl Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::MissingProperty(property_name) => {
                write!(f, "missing property: '{property_name}'")
            }
            AnalysisError::IncorrectType(expected_type_name, actual_type_name) => write!(
                f,
                "expected value to be of type '{expected_type_name}', but found it to be of type '{actual_type_name}'!"
            ),
            AnalysisError::IncorrectValueType(
                expected_type_name,
                actual_value,
                actual_type_name,
            ) => write!(
                f,
                "expected value ({expected_type_name}) to be of type '{actual_value}', but found it to be of type '{actual_type_name}'!"
            ),
            AnalysisError::MissingItems(expected_amount, acutal_amount) => write!(
                f,
                "Expected {expected_amount} items, but found {acutal_amount} instead!"
            ),
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl TryFrom<serde_json::Value> for Coordinate {
    type Error = AnalysisError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(mut array) => {
                if array.len() != 2 {
                    return Err(AnalysisError::MissingItems(3, array.len()));
                }
                Ok(Self {
                    latitude: array
                        .pop()
                        .filter(|v| matches!(v, Value::Number(_)))
                        .expect("'latitude' must exist")
                        .as_f64()
                        .expect("'latitude' must exist"),
                    longitude: array
                        .pop()
                        .filter(|v| matches!(v, Value::Number(_)))
                        .expect("'longitude' must exist")
                        .as_f64()
                        .expect("'longitude' must exist"),
                })
            }
            _ => Err(AnalysisError::IncorrectType(
                "Value::Array".to_string(),
                "Any other type".to_string(),
            )),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Line {
    color: String,
    id: i64,
    name: String,
    stroke: String,
    text_color: String,
}

impl TryFrom<Value> for Line {
    type Error = AnalysisError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match &value.as_object() {
            Some(object) => Ok(Self {
                color: object.extract("color")?,
                id: object.extract("id")?,
                name: object.extract("name")?,
                stroke: object.extract("stroke")?,
                text_color: object.extract("text_color")?,
            }),
            None => Err(AnalysisError::IncorrectType(
                "Value::Object".to_string(),
                type_name_of_val(&value).to_string(),
            )),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Train {
    delay: Option<String>,
    has_journey: bool,
    has_realtime: bool,
    has_realtime_journey: bool,
    line: Option<Line>,
    operator_provides_realtime_journey: String,
    original_line: Option<String>,
    original_rake: Option<String>,
    original_train_number: i32,
    position_correction: i32,
    rake: Option<String>,
    raw_coordinates: Option<Coordinate>,
    // raw_time
    ride_state: Option<String>,
    // route_identfier: Option<String>,
    state: Option<String>, // TODO: replace with enum when all states are known
    tenant: String,        // Most probably always `sbm`
    // time_intervals
    // time_since_update
    // timestamp
    train_id: String,
    train_number: Option<i64>,
    transmitting_vehicle: Option<String>, // Might be a unique id
    // type: String, // Can only be rail
    vehicle_number: Option<String>, // Might be a unique id
}

trait Extractor<T> {
    type Error;
    fn extract(&self, field_name: &str) -> Result<T, Self::Error>;
}

impl Extractor<String> for &Map<String, Value> {
    type Error = AnalysisError;
    fn extract(&self, field_name: &str) -> Result<String, Self::Error> {
        match self.get(field_name) {
            Some(property) => match property {
                Value::String(value) => Ok(value.to_string()),
                _ => {
                    println!("ERR TYP: {field_name}");
                    Err(AnalysisError::IncorrectValueType(
                        "String".to_string(),
                        format!("{property:?}"),
                        type_name_of_val(property).to_string(),
                    ))
                }
            },
            None => Err(AnalysisError::MissingProperty(field_name.to_string())),
        }
    }
}

impl Extractor<Option<String>> for &Map<String, Value> {
    type Error = AnalysisError;
    fn extract(&self, field_name: &str) -> Result<Option<String>, Self::Error> {
        match self.get(field_name) {
            Some(property) => match property {
                Value::String(value) => Ok(Some(value.to_string())),
                Value::Null => Ok(None),
                _ => Err(AnalysisError::IncorrectValueType(
                    "String".to_string(),
                    format!("{property:?}"),
                    type_name_of_val(&property).to_string(),
                )),
            },
            None => Ok(None),
        }
    }
}

impl Extractor<i64> for &Map<String, Value> {
    type Error = AnalysisError;
    fn extract(&self, field_name: &str) -> Result<i64, Self::Error> {
        match self.get(field_name) {
            Some(property) => match property {
                Value::Number(value) => Ok(value.as_i64().expect("must be integer")),
                _ => {
                    println!("ERR TYP: {field_name}");
                    Err(AnalysisError::IncorrectValueType(
                        "i64".to_string(),
                        format!("{property:?}"),
                        type_name_of_val(property).to_string(),
                    ))
                }
            },
            None => Err(AnalysisError::MissingProperty(field_name.to_string())),
        }
    }
}

impl Extractor<Option<i64>> for &Map<String, Value> {
    type Error = AnalysisError;
    fn extract(&self, field_name: &str) -> Result<Option<i64>, Self::Error> {
        match self.get(field_name) {
            Some(property) => match property {
                Value::Number(value) => Ok(value.as_i64()),
                Value::Null => Ok(None),
                _ => Err(AnalysisError::IncorrectValueType(
                    "Option<i64>".to_string(),
                    format!("{property:?}"),
                    type_name_of_val(property).to_string(),
                )),
            },
            None => Ok(None),
        }
    }
}

impl TryFrom<Content> for Train {
    type Error = AnalysisError;

    fn try_from(value: Content) -> Result<Self, Self::Error> {
        match value {
            Content::TrajectorySchematic(raw_train) => match raw_train {
                GeoJson::Feature(feature) => match &feature.properties {
                    Some(properties) => Ok(Self {
                        delay: properties.extract("delay")?,
                        has_journey: properties.get("has_journey").map_or(false, |v| match v {
                            Value::Bool(b) => *b,
                            _ => false,
                        }),
                        has_realtime: properties.get("has_realtime").map_or(false, |v| match v {
                            Value::Bool(b) => *b,
                            _ => false,
                        }),
                        has_realtime_journey: properties.get("has_realtime_journey").map_or(
                            false,
                            |v| match v {
                                Value::Bool(b) => *b,
                                _ => false,
                            },
                        ),
                        line: properties
                            .get("line")
                            .filter(|l| !l.is_null())
                            .map(|l| l.to_owned().try_into())
                            .transpose()?,
                        operator_provides_realtime_journey: properties
                            .extract("operator_provides_realtime_journey")?,
                        original_line: properties.extract("original_line")?,
                        original_rake: properties.extract("original_rake")?,
                        original_train_number: 0,
                        position_correction: 0,
                        rake: properties.extract("rake")?,
                        raw_coordinates: properties
                            .get("raw_coordinates")
                            .map(|x| x.clone().try_into().unwrap()),
                        ride_state: properties.extract("ride_state")?,
                        // route_identfier: properties.extract("route_identfier")?,
                        state: properties.extract("state")?,
                        tenant: properties.extract("tenant")?,
                        train_id: properties.extract("train_id")?,
                        train_number: properties.extract("train_number")?,
                        transmitting_vehicle: properties.extract("transmitting_vehicle")?,
                        vehicle_number: properties.extract("vehicle_number")?,
                    }),
                    None => Err(AnalysisError::IncorrectType(
                        "Properties".to_string(),
                        type_name_of_val(&feature.properties).to_string(),
                    )),
                },
                _ => Err(AnalysisError::IncorrectType(
                    "Feature".to_string(),
                    type_name_of_val(&raw_train).to_string(),
                )),
            },
            _ => Err(AnalysisError::IncorrectType(
                "TrajectorySchematic".to_string(),
                type_name_of_val(&value).to_string(),
            )),
        }
    }
}

#[derive(Debug)]
struct Counter<T>(HashMap<T, usize>);

impl<T> Counter<T> {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn insert(&mut self, k: T) -> Option<usize>
    where
        T: Eq + Hash,
    {
        match self.0.get_mut(&k) {
            Some(v) => {
                *v += 1;
                Some(*v)
            }
            None => self.0.insert(k, 1),
        }
    }
}

impl<T> Deref for Counter<T> {
    type Target = HashMap<T, usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
enum ColorConversionError {
    WrongBeginning,
    ParseInt(std::num::ParseIntError),
}

impl Display for ColorConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorConversionError::WrongBeginning => write!(f, "does not start with '#'"),
            ColorConversionError::ParseInt(err) => write!(f, "{err:?}"),
        }
    }
}

impl Error for ColorConversionError {}

impl From<std::num::ParseIntError> for ColorConversionError {
    fn from(value: std::num::ParseIntError) -> Self {
        ColorConversionError::ParseInt(value)
    }
}

fn try_color_from_string(s: String) -> Result<Color, ColorConversionError> {
    if !s.starts_with('#') {
        return Err(ColorConversionError::WrongBeginning);
    }

    let r: u8 = u8::from_str_radix(&s[1..3], 16)?;
    let g: u8 = u8::from_str_radix(&s[3..5], 16)?;
    let b: u8 = u8::from_str_radix(&s[5..7], 16)?;

    Ok(Color {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: 1.0,
    })
}

fn map(value: f32, a_min: f32, a_max: f32, b_min: f32, b_max: f32) -> f32 {
    (value - a_min) / (a_max - a_min) * (b_max - b_min) + b_min
}

enum State {
    Driving,
    Boarding,
}

impl From<String> for State {
    fn from(value: String) -> Self {
        if value == "DRIVING" {
            Self::Driving
        } else {
            Self::Boarding
        }
    }
}

#[allow(unused)]
struct Record {
    timestamp: String,
    position: Coordinate,
    line: String,
    line_color: Color,
    state: State,
    vehicle_number: String,
    train_number: i64,
}

impl Record {
    fn render(&self) {
        draw_circle(
            map(
                self.position.longitude as f32,
                11.0,
                12.0,
                0.0,
                screen_width(),
            ),
            map(
                self.position.latitude as f32,
                47.5,
                48.5,
                screen_height(),
                0.0,
            ),
            5.0,
            self.line_color,
        );
    }
}

impl TryFrom<ResponseMessage> for Record {
    type Error = AnalysisError;

    fn try_from(value: ResponseMessage) -> Result<Self, Self::Error> {
        match value.content {
            Content::TrajectorySchematic(trajectory) => match trajectory {
                GeoJson::Feature(feature) => match feature.properties {
                    Some(properties) => Ok(Self {
                        timestamp: value.timestamp.to_string(),
                        position: properties
                            .get("raw_coordinates")
                            .map(std::borrow::ToOwned::to_owned)
                            .ok_or(AnalysisError::MissingProperty(
                                "raw_coordinates".to_string(),
                            ))?
                            .try_into()?,
                        line: properties
                            .get("line")
                            .map(std::borrow::ToOwned::to_owned)
                            .ok_or(AnalysisError::MissingProperty("line".to_string()))?
                            .as_object()
                            .ok_or(AnalysisError::IncorrectType(
                                "object".to_string(),
                                "unknown".to_string(),
                            ))?
                            .get("name")
                            .ok_or(AnalysisError::MissingProperty("name".to_string()))?
                            .as_str()
                            .ok_or(AnalysisError::IncorrectType(
                                "string".to_string(),
                                "unknown".to_string(),
                            ))?
                            .to_string(),
                        line_color: try_color_from_string(
                            properties
                                .get("line")
                                .map(std::borrow::ToOwned::to_owned)
                                .ok_or(AnalysisError::MissingProperty("line".to_string()))?
                                .as_object()
                                .ok_or(AnalysisError::IncorrectType(
                                    "object".to_string(),
                                    "unknown".to_string(),
                                ))?
                                .get("color")
                                .ok_or(AnalysisError::MissingProperty("color".to_string()))?
                                .as_str()
                                .ok_or(AnalysisError::IncorrectType(
                                    "string".to_string(),
                                    "unknown".to_string(),
                                ))?
                                .to_string(),
                        )
                        .map_err(|cce| {
                            AnalysisError::IncorrectType("Color".to_string(), cce.to_string())
                        })?,
                        state: properties
                            .get("state")
                            .map(std::borrow::ToOwned::to_owned)
                            .ok_or(AnalysisError::MissingProperty("state".to_string()))?
                            .as_str()
                            .ok_or(AnalysisError::MissingProperty("state".to_string()))?
                            .to_string()
                            .into(),
                        vehicle_number: properties
                            .get("vehicle_number")
                            .map(std::borrow::ToOwned::to_owned)
                            .ok_or(AnalysisError::MissingProperty("vehicle_number".to_string()))?
                            .as_str()
                            .ok_or(AnalysisError::MissingProperty("vehicle_number".to_string()))?
                            .to_string(),
                        train_number: properties
                            .get("train_number")
                            .map(std::borrow::ToOwned::to_owned)
                            .ok_or(AnalysisError::MissingProperty("train_number".to_string()))?
                            .as_i64()
                            .ok_or(AnalysisError::IncorrectType(
                                "i64".to_string(),
                                "unknown".to_string(),
                            ))?,
                    }),
                    None => Err(AnalysisError::MissingProperty("properties".to_string())),
                },
                _ => Err(AnalysisError::IncorrectType(
                    "GeoJson::Feature".to_string(),
                    type_name_of_val(&trajectory).to_string(),
                )),
            },
            Content::DeletedVehiclesSchematic(_) => todo!(),
            _ => Err(AnalysisError::IncorrectType(
                "Content::TrajectorySchematic".to_string(),
                type_name_of_val(&value.content).to_string(),
            )),
        }
    }
}

struct Vehicle {
    number: String,
    records: Vec<Record>,
}

impl Vehicle {
    fn from_record(r: Record) -> Self {
        Self {
            number: r.vehicle_number.clone(),
            records: vec![r],
        }
    }

    fn update(&mut self, r: Record) {
        self.records.push(r);
    }

    fn render(&self, idx: usize) {
        if let Some(record) = self.records.get(idx) {
            record.render();
        }
    }
}

struct Trains(HashMap<String, Vehicle>);

impl Trains {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn insert(&mut self, r: Record) {
        match self.get_mut(&r.vehicle_number) {
            Some(v) => v.update(r),
            None => {
                self.0
                    .insert(r.vehicle_number.clone(), Vehicle::from_record(r));
            }
        }
    }

    fn render(&self, idx: usize) {
        for t in self.0.values() {
            t.render(idx);
        }
    }
}

impl Deref for Trains {
    type Target = HashMap<String, Vehicle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Trains {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let reader =
        BufReader::new(File::open("./s-bahn-munich-live-map.jsonl").expect("Cannot open file.txt"));
    let mut trains: usize = 0;
    let mut delays: Counter<Option<String>> = Counter::new();
    let mut states: Counter<Option<String>> = Counter::new();
    let mut ride_states: Counter<Option<String>> = Counter::new();
    let mut original_lines: Counter<Option<String>> = Counter::new();
    let mut lines: Counter<Option<Line>> = Counter::new();
    let mut persistent_trains = Trains::new();

    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !line.is_empty() {
                    match serde_json::from_str::<ResponseMessage>(&line) {
                        Ok(m) => {
                            match m.content {
                                Content::TrajectorySchematic(_) => {
                                    match <Content as TryInto<Train>>::try_into(m.content.clone()) {
                                        Ok(train) => {
                                            trains += 1;
                                            delays.insert(train.delay.clone());
                                            states.insert(train.state.clone());
                                            ride_states.insert(train.ride_state.clone());
                                            original_lines.insert(train.original_line.clone());
                                            lines.insert(train.line.clone());
                                            // persistent_trains.insert(train.clone());
                                        }
                                        Err(err) => {
                                            eprintln!("should be train: {err}\n\t{line:#?}");
                                        }
                                    };
                                    match <ResponseMessage as TryInto<Record>>::try_into(m.clone()) {
                                        Ok(record) => {
                                            persistent_trains.insert(record);
                                        }
                                        Err(_err) => {
                                            // eprintln!("should be record: {}\n\t{:#?}", err, line)
                                        }
                                    }
                                    // break;
                                }
                                // Content::SbmNewsTicker(news) => {
                                //     println!("{:#?}", news);
                                // },
                                Content::StationSchematic(_station) => {}
                                _ => {}
                            }
                        }
                        Err(err) => eprintln!("{err}, unable to parse: '{line}'"),
                    }
                }
            }
            Err(err) => eprintln!("ERROR: {err}"),
        };
    }
    println!("trains: {trains}");
    println!("delays: {delays:#?}");
    println!("states: {states:#?}");
    println!("ride_states: {ride_states:#?}");
    println!("original_line: {original_lines:#?}");
    println!("line: {lines:#?}");

    // Render
    let mut i = 0;
    loop {
        clear_background(Color::from_hex(0x009E_9E9E));

        persistent_trains.render(i);

        next_frame().await;
        thread::sleep(Duration::from_millis(20));
        i += 1;
        if i >= 995 {
            i = 0;
        }
    }
}
