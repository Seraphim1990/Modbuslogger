use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodingType {
    pub id: i64,
    #[sqlx(rename = "uiName")]
    pub ui_name: String,
    #[sqlx(rename = "programName")]
    pub program_name: String,
}

#[derive(Debug, Serialize, FromRow, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QmlDecodingAddons {
    pub id: i64,
    #[sqlx(rename = "parentDescriptionType")]
    pub parent_type: i32,
    #[sqlx(rename = "textForQML")]
    pub tex_for_qml: String,
}