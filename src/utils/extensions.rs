use schemars::_serde_json::{Value, Map};

pub trait ValueExt {
    fn to_string(&self) -> String;
    fn to_bool(&self) -> bool;
    fn to_i64(&self) -> i64;
    fn to_u64(&self) -> u64;
    fn to_f64(&self) -> f64;
}

impl ValueExt for Option<&Value> {
    fn to_string(&self) -> String {
        self.unwrap().as_str().unwrap().to_string()
    }
    fn to_bool(&self) -> bool {
        self.unwrap().as_bool().unwrap()
    }
    fn to_i64(&self) -> i64 {
        self.unwrap().as_i64().unwrap()
    }
    fn to_u64(&self) -> u64 {
        self.unwrap().as_u64().unwrap()
    }
    fn to_f64(&self) -> f64 {
        self.unwrap().as_f64().unwrap()
    }
}
