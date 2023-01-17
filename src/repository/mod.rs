#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("cannot find `{id}:{entity}` in the following {method}.")]
    NotFound {
        method: &'static str,
        entity: &'static str,
        id: String
    },
    #[error(transparent)]
    External(anyhow::Error)
}