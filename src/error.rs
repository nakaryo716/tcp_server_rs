use thiserror::Error;

#[derive(Debug, Error)]
#[error("error {0:?}")]
pub struct ServiceError(Box<dyn std::error::Error>);

impl ServiceError {
    pub fn new<E: std::error::Error + 'static>(e: E) -> Self {
        ServiceError(Box::new(e))
    }
}
