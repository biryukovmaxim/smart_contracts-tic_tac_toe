use crate::board::Player;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum GameStatus {
    NotStarted,
    PlayerTurning(Player),
    PlayerWin(Player),
    Draw,
}

impl From<GameStatus> for u8 {
    fn from(gs: GameStatus) -> Self {
        match gs {
            GameStatus::NotStarted => 0,
            GameStatus::PlayerTurning(Player::XPlayer) => 1,
            GameStatus::PlayerTurning(Player::OPlayer) => 2,
            GameStatus::PlayerWin(Player::XPlayer) => 3,
            GameStatus::PlayerWin(Player::OPlayer) => 4,
            GameStatus::Draw => 5,
        }
    }
}

impl From<u8> for GameStatus {
    fn from(num: u8) -> Self {
        match num {
            0 => Self::NotStarted,
            1 => Self::PlayerTurning(Player::XPlayer),
            2 => Self::PlayerTurning(Player::OPlayer),
            3 => Self::PlayerWin(Player::XPlayer),
            4 => Self::PlayerWin(Player::OPlayer),
            _ => Self::Draw,
        }
    }
}
