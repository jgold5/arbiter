use serde::{Deserialize, Serialize};

/// All errors related to the instrument
#[derive(Debug)]
pub enum InstrumentError {
    EmptyIdentifier,
}

/// Identifier of instruments used in order book
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstrumentId(String);

impl InstrumentId {
    pub fn new(id: String) -> Result<InstrumentId, InstrumentError> {
        if id.is_empty() {
            Err(InstrumentError::EmptyIdentifier)
        } else {
            Ok(InstrumentId(id))
        }
    }
}