use std::str::FromStr;
use std::sync::OnceLock;

use eyre::eyre;
use fluvio_smartmodule::dataplane::smartmodule::{SmartModuleExtraParams, SmartModuleInitError};
use fluvio_smartmodule::{RecordData, Result, SmartModuleRecord, smartmodule};
use serde_json::Value;

static KEY: OnceLock<String> = OnceLock::new();
static DELETE: OnceLock<bool> = OnceLock::new();

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let string = std::str::from_utf8(record.value.as_ref())?;
    let mut value = Value::from_str(string)?;
    let obj = value
        .as_object_mut()
        .ok_or(eyre!("Failed to parse value"))?;

    let field_key = KEY.get().expect("Invalid state");
    let record_key = if DELETE.get().is_some() {
        obj.remove(field_key)
            .ok_or(eyre!(format!("Key missing in record")))?
            .to_string()
    } else {
        obj.get(field_key)
            .ok_or(eyre!("Field missing in record"))?
            .to_string()
    };

    Ok((Some(record_key.into()), value.to_string().as_str().into()))
}

#[smartmodule(init)]
fn init(params: SmartModuleExtraParams) -> Result<()> {
    if params.get("delete").is_some() {
        DELETE
            .set(true)
            .map_err(|_| eyre!("Failed to set input param"))?;
    }

    if let Some(key) = params.get("key") {
        KEY.set(key.clone())
            .map_err(|err| eyre!("failed setting key: {:#?}", err))
    } else {
        Err(SmartModuleInitError::MissingParam("key".to_string()).into())
    }
}
