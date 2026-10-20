use crate::{AddressError, BranchCoordinate, NodeId};

pub const MAX_NODE_ADDRESS: usize = 14;
pub const MAX_BRANCH_ADDRESS: usize = 28;

fn encode(mut value: u64) -> String {
    let mut buffer = [b'0'; 13];
    let mut start = buffer.len();
    loop {
        start -= 1;
        let digit = (value % 36) as u8;
        buffer[start] = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + digit - 10
        };
        value /= 36;
        if value == 0 {
            break;
        }
    }
    // Every byte above is an ASCII digit/letter.
    buffer[start..].iter().map(|b| char::from(*b)).collect()
}

fn number(text: &str) -> Result<u64, AddressError> {
    if text.is_empty()
        || text.len() > 13
        || (text.len() > 1 && text.starts_with('0'))
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
    {
        return Err(AddressError::InvalidSyntax);
    }
    u64::from_str_radix(text, 36).map_err(|_| AddressError::Overflow)
}

impl NodeId {
    pub fn short_address(self) -> String {
        format!("n{}", encode(self.get()))
    }

    /// Syntax/range validation only; a session query must still check scope
    /// and existence. Rejects uppercase, leading zeroes and noncanonical text.
    pub fn parse_address(text: &str) -> Result<Self, AddressError> {
        if text.len() > MAX_NODE_ADDRESS {
            return Err(AddressError::TooLong);
        }
        let digits = text.strip_prefix('n').ok_or(AddressError::InvalidSyntax)?;
        Self::new(number(digits)?).ok_or(AddressError::ZeroIdentity)
    }
}

impl BranchCoordinate {
    pub fn short_address(self) -> String {
        format!("b{}:{}", encode(self.branch()), encode(self.ordinal()))
    }

    pub fn parse_address(text: &str) -> Result<Self, AddressError> {
        if text.len() > MAX_BRANCH_ADDRESS {
            return Err(AddressError::TooLong);
        }
        let text = text.strip_prefix('b').ok_or(AddressError::InvalidSyntax)?;
        let (branch, ordinal) = text.split_once(':').ok_or(AddressError::InvalidSyntax)?;
        Self::new(number(branch)?, number(ordinal)?).ok_or(AddressError::ZeroIdentity)
    }
}
