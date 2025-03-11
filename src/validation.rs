pub trait Validation {
    fn validate(&self) -> bool;
}

pub fn validated<T: Validation>(v: Option<T>) -> Option<T> {
    if let Some(v) = v {
        if v.validate() {
            Some(v)
        } else {
            None
        }
    } else {
        None
    }
}
