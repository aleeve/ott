use eyre::eyre;
use fluvio_smartmodule::{RecordData, Result, SmartModuleRecord, smartmodule};
use serde_json::{Map, Value};

#[smartmodule(filter_map)]
pub fn filter_map(record: &SmartModuleRecord) -> Result<Option<(Option<RecordData>, RecordData)>> {
    let key = record.key.clone();

    let string = std::str::from_utf8(record.value.as_ref())?;
    let mut value: Value = serde_json::from_str(string)?;
    let obj = value
        .as_object_mut()
        .ok_or(eyre!("Failed to parse value"))?;

    if let Ok(uri) = get_uri(obj) {
        let uri_value = Value::String(uri);
        obj.insert("uri".to_string(), uri_value);

        Ok(Some((key, value.to_string().as_str().into())))
    } else {
        Ok(None)
    }
}

fn get_uri(obj: &Map<String, Value>) -> Result<String> {
    let did = obj
        .get("did")
        .and_then(|v| v.as_str())
        .ok_or(eyre!("did missing or not a string"))?;

    let commit = obj.get("commit").ok_or(eyre!("commit missing"))?;

    let collection = commit
        .get("collection")
        .and_then(|v| v.as_str())
        .ok_or(eyre!("commit.collection missing or not a string"))?;

    let rkey = commit
        .get("rkey")
        .and_then(|v| v.as_str())
        .ok_or(eyre!("commit.rkey missing or not a string"))?;

    Ok(format!("at://{did}/{collection}/{rkey}"))
}
