use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FormValue {
    String(String),
    Boolean(bool),
}

pub type FormState = HashMap<String, FormValue>;
