use std::str::FromStr;
use std::sync::OnceLock;

use eyre::eyre;
use fluvio_smartmodule::dataplane::smartmodule::{SmartModuleExtraParams, SmartModuleInitError};
use fluvio_smartmodule::{RecordData, Result, SmartModuleRecord, smartmodule};
use serde_json::Value;

static CRITERIA: OnceLock<String> = OnceLock::new();

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let string = std::str::from_utf8(record.value.as_ref())?;
    let mut value = Value::from_str(string)?;
    let obj = value
        .as_object_mut()
        .ok_or(eyre!("Failed to parse value"))?;
    let key = obj
        .remove(CRITERIA.get().ok_or(eyre!("Invalid state"))?)
        .ok_or(eyre!("Key missing in record"))?;

    Ok((
        Some(key.to_string().into()),
        value.to_string().as_str().into(),
    ))
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    if let Some(key) = params.get("key") {
        CRITERIA
            .set(key.clone())
            .map_err(|err| eyre!("failed setting key: {:#?}", err))
    } else {
        Err(SmartModuleInitError::MissingParam("key".to_string()).into())
    }
}
