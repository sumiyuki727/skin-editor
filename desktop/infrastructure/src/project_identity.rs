//! OS entropy adapter; the application owns admission and collision checks.
use skin_editor_application::{HistoryIdentitySource, IdentityError};

#[derive(Default)]
pub struct RandomHistoryIdentities;
impl HistoryIdentitySource for RandomHistoryIdentities {
    fn next_identity(&mut self) -> Result<u128, IdentityError> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| IdentityError::Unavailable)?;
        Ok(u128::from_be_bytes(bytes))
    }
}
