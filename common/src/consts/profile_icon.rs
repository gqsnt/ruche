use std::fmt::Formatter;
use bitcode::{Decode, Encode};

#[derive(Copy, Clone, Default, Encode,Decode)]
pub struct ProfileIcon(pub u16);

impl std::fmt::Display for ProfileIcon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
