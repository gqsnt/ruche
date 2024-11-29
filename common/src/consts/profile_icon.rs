use std::fmt::Formatter;

#[derive(Copy, Clone, Default)]
pub struct ProfileIcon(pub u16);

impl std::fmt::Display for ProfileIcon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
