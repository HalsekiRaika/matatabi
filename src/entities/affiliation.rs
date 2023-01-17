#[derive(Debug, Clone, Copy)]
pub struct AffiliationId(i64);

impl AffiliationId {
    pub fn new(id: impl Into<i64>) -> Self {
        Self(id.into())
    }
}

impl AsRef<i64> for AffiliationId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl From<i64> for AffiliationId {
    fn from(prime: i64) -> Self {
        Self(prime)
    }
}

impl From<AffiliationId> for i64 {
    fn from(id: AffiliationId) -> Self {
        id.0
    }
}

#[derive(Debug, destructure::Destructure)]
pub struct Affiliation {
    id: AffiliationId,
    name: String,
}

impl Affiliation {
    pub fn new(id: impl Into<AffiliationId>, name: impl Into<String>) -> Self {
        Self { id: id.into(), name: name.into() }
    }

    pub fn id(&self) -> &AffiliationId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}