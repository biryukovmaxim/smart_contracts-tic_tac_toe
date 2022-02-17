use crate::errors::Error;
use core::ops::Deref;
use core::ops::DerefMut;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum Player {
    XPlayer,
    OPlayer,
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum Mark {
    X,
    O,
}

impl Mark {
    pub fn is_x(&self) -> bool {
        matches!(self, Self::X)
    }

    pub fn is_o(&self) -> bool {
        matches!(self, Self::O)
    }
}

impl From<Mark> for u8 {
    fn from(m: Mark) -> Self {
        match m {
            Mark::X => 0,
            Mark::O => 1,
        }
    }
}

impl From<u8> for Mark {
    fn from(num: u8) -> Self {
        match num {
            0 => Mark::X,
            _ => Mark::O,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct Board<T: From<Mark> + Into<Mark> + Copy, const LEN: usize>([Option<T>; LEN]);

impl<T: From<Mark> + Into<Mark> + Copy, const LEN: usize> Board<T, LEN> {
    pub fn new(arr: [Option<T>; LEN]) -> Self {
        Self(arr)
    }
    pub fn default() -> Self {
        Self([None; LEN])
    }

    pub fn turn(&mut self, p: Player, coordinate: usize) -> Result<(), Error> {
        match self.get_mut(coordinate) {
            None => Err(Error::CoordinateNotExists),
            Some(m) => {
                if m.is_none() {
                    match p {
                        Player::XPlayer => *m = Some(T::from(Mark::X)),
                        Player::OPlayer => *m = Some(T::from(Mark::O)),
                    }
                    Ok(())
                } else {
                    Err(Error::CoordinateAlreadyFilled)
                }
            }
        }
    }
}

impl<T: From<Mark> + Into<Mark> + Copy, const LEN: usize> Deref for Board<T, LEN> {
    type Target = [Option<T>; LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: From<Mark> + Into<Mark> + Copy, const LEN: usize> DerefMut for Board<T, LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Player};
    use crate::errors::Error;

    #[test]
    fn turns() {
        let mut state: Board<u8, 9> = Board::default();
        let bad_coordinate = state.turn(Player::XPlayer, 10);
        assert_eq!(bad_coordinate, Err(Error::CoordinateNotExists));

        let first_turn = state.turn(Player::XPlayer, 0);
        assert_eq!(first_turn, Ok(()));

        let filled_turn = state.turn(Player::OPlayer, 0);
        assert_eq!(filled_turn, Err(Error::CoordinateAlreadyFilled));
    }
}
