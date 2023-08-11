use dotenvy::{self, dotenv};
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::SystemTime;

struct InfluxCredentials {
    url: String,
    token: String,
    org: String,
    bucket: String,
    host: String,
}

struct InfluxTag {
    name: String,
    value: String,
}

struct InfluxField {
    name: String,
    value: InfluxFieldValue,
}

enum InfluxFieldValue {
    Float(f64),
    Int(i64),
    String(String),
}

impl fmt::Display for InfluxFieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{}", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
        }
    }
}

struct Backup {
    host: String,
    backup_name: String,
    encryption: String,
    repo_location: String,
    duration: f64,
    compressed_size: i64,
    deduplicated_size: i64,
    number_of_files: i64,
    original_size: i64,
}

struct InfluxPoint {
    measurement: String,
    tags: Vec<InfluxTag>,
    fields: Vec<InfluxField>,
}

fn main() {}

fn read_env_file() -> InfluxCredentials {
    dotenv().ok();

    InfluxCredentials {
        url: env::var("INFLUX_URL").expect("INFLUX_URL missing"),
        token: env::var("INFLUX_TOKEN").expect("INFLUX_TOKEN missing"),
        org: env::var("INFLUX_ORG").expect("INFLUX_ORG missing"),
        bucket: env::var("INFLUX_BUCKET").expect("INFLUX_BUCKET missing"),
        host: env::var("HOST").expect("HOST missing"),
    }
}

fn read_borg_json_file(file_path: &Path) -> Result<Value, Box<dyn Error>> {
    let json_file = File::open(file_path)?;
    let file_reader = BufReader::new(json_file);

    let json_value: Value = serde_json::from_reader(file_reader)?;

    return Ok(json_value);
}

fn extract_data_from_json(json_data: Value, host: String) -> Backup {
    Backup {
        host: host,
        backup_name: json_data["archive"]["name"].to_string(),
        encryption: json_data["encryption"]["mode"].to_string(),
        repo_location: json_data["repository"]["location"].to_string(),
        duration: json_data["archive"]["duration"].as_f64().unwrap(),
        compressed_size: json_data["archive"]["stats"]["compressed_size"]
            .as_i64()
            .unwrap(),
        deduplicated_size: json_data["archive"]["deduplicated_size"].as_i64().unwrap(),
        number_of_files: json_data["archive"]["stats"]["nfiles"].as_i64().unwrap(),
        original_size: json_data["archive"]["stats"]["original_size"]
            .as_i64()
            .unwrap(),
    }
}

fn create_influx_point_from_backup(backup: Backup) -> InfluxPoint {
    let tags: Vec<InfluxTag> = vec![
        InfluxTag {
            name: "host".to_string(),
            value: backup.host,
        },
        InfluxTag {
            name: "backup_name".to_string(),
            value: backup.backup_name,
        },
        InfluxTag {
            name: "encryption".to_string(),
            value: backup.encryption,
        },
        InfluxTag {
            name: "repo_location".to_string(),
            value: backup.repo_location,
        },
    ];
    let fields: Vec<InfluxField> = vec![
        InfluxField {
            name: "duration".to_string(),
            value: InfluxFieldValue::Float(backup.duration),
        },
        InfluxField {
            name: "compressed_size".to_string(),
            value: InfluxFieldValue::Int(backup.compressed_size),
        },
        InfluxField {
            name: "deduplicated_size".to_string(),
            value: InfluxFieldValue::Int(backup.deduplicated_size),
        },
        InfluxField {
            name: "number_of_files".to_string(),
            value: InfluxFieldValue::Int(backup.number_of_files),
        },
        InfluxField {
            name: "original_size".to_string(),
            value: InfluxFieldValue::Int(backup.original_size),
        },
    ];

    InfluxPoint {
        measurement: "backup".to_string(),
        tags: tags,
        fields: fields,
    }
}

fn build_raw_data_from_point(point: InfluxPoint) -> String {
    let mut raw_data = point.measurement;

    for tag in point.tags {
        raw_data = format!("{}, {}={}", raw_data, tag.name, tag.value);
    }

    for field in point.fields {
        raw_data = format!("{} {}={}", raw_data, field.name, field.value);
    }

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!("{} {}", raw_data, timestamp)
}

fn write_to_influx(point: InfluxPoint, credentials: InfluxCredentials) {
    let client = reqwest::blocking::Client::new();
    let response = client.post(credentials.url).body("DATA").send();
}