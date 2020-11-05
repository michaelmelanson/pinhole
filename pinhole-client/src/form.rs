use pinhole_protocol::form_state::{FormState as RemoteFormState, FormValue as RemoteFormValue};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum LocalFormValue {
    String(String),
    Boolean(bool),
}

impl log::kv::ToValue for LocalFormValue {
    fn to_value(&self) -> log::kv::Value {
        log::kv::Value::from_debug(self)
    }
}

impl LocalFormValue {
    pub fn boolean(&self) -> bool {
        match self {
            LocalFormValue::Boolean(b) => *b,
            _ => false,
        }
    }

    pub fn string(&self) -> String {
        match self {
            LocalFormValue::String(s) => s.clone(),
            _ => "".to_string(),
        }
    }
}

pub type LocalFormState = HashMap<String, LocalFormValue>;
pub fn convert_form_state(local_state: &LocalFormState) -> RemoteFormState {
    let mut remote_state: RemoteFormState = HashMap::new();

    for (key, value) in local_state {
        remote_state.insert(
            key.clone(),
            match value {
                LocalFormValue::String(s) => RemoteFormValue::String(s.to_string()),
                LocalFormValue::Boolean(b) => RemoteFormValue::Boolean(*b),
            },
        );
    }

    remote_state
}
