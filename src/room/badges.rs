use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Condition {
    id: String,
    trigger: String,
    value: String,
    values: Vec<String>,
    time_trial: bool,
    disabled: bool,
    #[serde(flatten)]
    inner: InnerCondition,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum InnerCondition {
    MapArea {
        map: isize,
        map_x_1: i16,
        map_y_1: i16,
        map_x_2: i16,
        map_y_2: i16,
    },
    SingleSwitch {
        id: isize,
        value: isize,
        delay: isize,
    },
    MultipleSwitches {
        ids: Vec<isize>,
        values: Vec<isize>,
        delay: isize,
    },
    SingleVariable {
        id: isize,
        value: isize,
        value2: isize,
        op: String,
        delay: isize,
    },
    MultipleVariables {
        ids: Vec<isize>,
        values: Vec<isize>,
        ops: Vec<String>,
        delay: isize,
        trigger: bool,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    group: String,
    order: isize,
    map_order: isize,
    bp: isize,
    req_type: String,
    req_int: isize,
    #[serde(flatten)]
    req: Requirement,
    req_count: isize,
    map: isize,
    map_x: i16,
    map_y: i16,
    secret: bool,
    secret_map: bool,
    secret_condition: bool,
    hidden: bool,
    parent: String,
    overlay_type: isize,
    art: String,
    #[serde(rename = "animated")]
    is_animated: bool,
    batch: isize,
    dev: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Requirement {
    #[serde(rename = "reqString")]
    SingleString(String),
    #[serde(rename = "reqStrings")]
    MultipleStrings(Vec<String>),
    #[serde(rename = "reqStringArrays")]
    StringArrays(Vec<Vec<String>>),
}
