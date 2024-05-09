use csv::Reader;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Player {
    name: String,
    position: String,
    #[serde(rename = "DOB")]
    dob: String,
    nationality: String,
    #[serde(rename = "Kit Number")]
    kit: u8,
}

pub fn process_csv(input: &str, output: &str, format: &str) -> anyhow::Result<()> {
    let mut reader = Reader::from_path(input)?;
    let mut ret = Vec::new();
    let headers = reader.headers()?.clone();
    for result in reader.records() {
        let record = result?;
        let json_value = headers.iter().zip(record.iter()).collect::<Value>();
        ret.push(json_value);
    }

    let content = match format {
        "json" => serde_json::to_string_pretty(&ret)?,
        "yaml" => serde_yaml::to_string(&ret)?,
        _ => return Err(anyhow::anyhow!("Invalid format")),
    };
    fs::write(output, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_csv_to_json() {
        let input = "assets/juventus.csv";
        let output = "output.json";
        let format = "json";
        process_csv(input, output, format).unwrap();
        let content = fs::read_to_string(output).unwrap();
        let players: Vec<Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(players.len(), 27);
        assert_eq!(players[0]["Name"], "Wojciech Szczesny");
        assert_eq!(players[1]["Position"], "Goalkeeper");
        assert_eq!(players[2]["DOB"], "Jan 28, 1978 (41)");
    }

    #[test]
    fn test_process_csv_to_yaml() {
        let input = "assets/juventus.csv";
        let output = "output.yaml";
        let format = "yaml";
        process_csv(input, output, format).unwrap();
        let content = fs::read_to_string(output).unwrap();
        let players: Vec<Value> = serde_yaml::from_str(&content).unwrap();
        assert_eq!(players.len(), 27);
        assert_eq!(players[0]["Name"], "Wojciech Szczesny");
        assert_eq!(players[1]["Position"], "Goalkeeper");
        assert_eq!(players[2]["DOB"], "Jan 28, 1978 (41)");
    }
}
