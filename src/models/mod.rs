pub mod affiliation;
pub mod channel;
pub mod error;
pub mod liver;
pub mod upcoming;

use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
#[serde(transparent)]
pub struct NumId<T> {
    value: i64,
    #[serde(skip)]
    _mark: std::marker::PhantomData<T>
}

#[allow(dead_code)]
impl<T> NumId<T> {
    pub fn new(id: i64) -> NumId<T> {
        Self { value: id, _mark: std::marker::PhantomData }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct StringId<T> {
    value: String,
    #[serde(skip)]
    _mark: std::marker::PhantomData<T>
}

#[allow(dead_code)]
impl<T> StringId<T> {
    pub fn new(id: impl Into<String>) -> StringId<T> {
        Self { value: id.into(), _mark: std::marker::PhantomData }
    }

    pub fn as_ref(&self) -> &str {
        &self.value
    }
}