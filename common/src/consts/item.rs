use std::fmt::Formatter;

#[derive(Copy, Clone, Default)]
pub struct Item(pub u32);



impl TryFrom<u32> for Item {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(())
        } else {
            Ok(Item(value))
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
