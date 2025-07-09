pub trait Validation {
    fn validate(&self) -> bool;
}

pub fn validated<T: Validation>(v: Option<T>) -> Option<T> {
    v.filter(|v| v.validate())
}
