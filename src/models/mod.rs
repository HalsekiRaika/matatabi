pub mod affiliation;
pub mod channel;
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
    pub fn new(id: impl Into<i64>) -> NumId<T> {
        Self { value: id.into(), _mark: std::marker::PhantomData }
    }
}

impl<T> From<NumId<T>> for i64 {
    fn from(inner: NumId<T>) -> Self {
        inner.value
    }
}

impl<T> From<i64> for NumId<T> {
    fn from(prime: i64) -> Self {
        Self::new(prime)
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

impl<T> From<StringId<T>> for String {
    fn from(inner: StringId<T>) -> Self {
        inner.value
    }
}

impl<T> From<String> for StringId<T> {
    fn from(prime: String) -> Self {
        Self::new(prime)
    }
}