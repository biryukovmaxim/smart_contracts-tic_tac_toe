#![feature(is_some_with)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub mod board;
pub mod errors;
pub mod game_status;

#[ink::contract]
mod tic_tac_toe {
    const LENGTH: usize = 3;
    const SIZE: usize = LENGTH * LENGTH;

    use crate::board::Player::{OPlayer, XPlayer};
    use crate::board::{Board, Mark, Player};
    use crate::errors::Error;
    use crate::game_status;
    use crate::game_status::GameStatus;

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    #[derive(Default)]
    pub struct TicTacToe {
        player_x_account: AccountId,
        player_o_account: Option<AccountId>,
        board: [Option<u8>; SIZE],
        game_status: u8,
    }

    impl TicTacToe {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let contract_account_id = Self::env().account_id();
            Self::env().emit_event(WaitingOpponent {
                contract_account_id,
                player_x: caller,
                player_0: None,
            });

            Self {
                player_x_account: caller,
                player_o_account: None,
                board: *Board::default(),
                game_status: GameStatus::NotStarted.into(),
            }
        }

        #[ink(constructor)]
        pub fn with_opponent(player_o_account: AccountId) -> Self {
            let caller = Self::env().caller();
            let contract_account_id = Self::env().account_id();

            Self::env().emit_event(WaitingOpponent {
                contract_account_id,
                player_x: caller,
                player_0: Some(player_o_account),
            });

            Self {
                player_x_account: Self::env().caller(),
                player_o_account: Some(player_o_account),
                board: *Board::default(),
                game_status: GameStatus::NotStarted.into(),
            }
        }
        #[ink(message)]
        pub fn join_game(&mut self) -> Result<()> {
            let game_status = GameStatus::from(self.game_status);
            let caller: AccountId = self.env().caller();

            if !matches!(game_status, GameStatus::NotStarted) {
                Err(Error::GameAlreadyStarted)
            } else if caller == self.player_x_account {
                Err(Error::ForGameNeedsAtLeast2Players)
            } else if matches!(game_status, GameStatus::NotStarted)
                && self
                    .player_o_account
                    .is_some_with(|second_player| *second_player != caller)
            {
                Err(Error::WaitingAnotherDefinedPlayer)
            } else {
                self.player_o_account = Some(caller);
                self.env().emit_event(GameStarted {
                    contract_account_id: Self::env().account_id(),
                    player_x: self.player_x_account,
                    player_o: caller,
                });
                let turning_player = Player::XPlayer;
                self.game_status = u8::from(GameStatus::PlayerTurning(turning_player));
                Ok(())
            }
        }
        #[ink(message)]
        pub fn turn(&mut self, coordinate: u8) -> Result<()> {
            let caller: AccountId = self.env().caller();
            let o_player: AccountId = self.player_o_account.ok_or(Error::GameNotStarted)?;
            let player = if caller == o_player {
                Ok(Player::OPlayer)
            } else if caller == self.player_x_account {
                Ok(Player::XPlayer)
            } else {
                Err(Error::UnknownPlayer)
            }?;

            let game_status = GameStatus::from(self.game_status);
            match game_status {
                GameStatus::NotStarted => Err(Error::GameNotStarted),
                GameStatus::PlayerWin(_) | GameStatus::Draw => Err(Error::GameAlreadyOver),
                GameStatus::PlayerTurning(turning_player) if turning_player != player => {
                    Err(Error::AnotherPlayerShouldTurn)
                }
                GameStatus::PlayerTurning(_) => {
                    let mut board = Board::new(self.board);
                    board.turn(player, coordinate as usize)?;
                    self.board = *board;
                    match Self::check_state(board, player, coordinate as usize) {
                        None => self.switch_player(),
                        Some(gs) => self.game_status = u8::from(gs),
                    };
                    Ok(())
                }
            }
        }

        /// Returns the player whose turn it is.
        #[ink(message)]
        pub fn get_turning_player(&self) -> Option<AccountId> {
            let game_status = game_status::GameStatus::from(self.game_status);
            match game_status {
                GameStatus::NotStarted | GameStatus::Draw | GameStatus::PlayerWin(_) => None,
                GameStatus::PlayerTurning(player) => match player {
                    Player::XPlayer => Some(self.player_x_account),
                    Player::OPlayer => Some(self.player_o_account.unwrap()),
                },
            }
        }

        /// Return current state of game board.
        #[ink(message)]
        pub fn get_board(&self) -> Board<u8, 9> {
            Board::new(self.board)
        }

        /// Return current state of game.
        #[ink(message)]
        pub fn get_game_status(&self) -> GameStatus {
            GameStatus::from(self.game_status)
        }

        fn switch_player(&mut self) {
            self.game_status = u8::from(match GameStatus::from(self.game_status) {
                GameStatus::NotStarted => unreachable!(),
                GameStatus::PlayerWin(_) => unreachable!(),
                GameStatus::Draw => unreachable!(),
                GameStatus::PlayerTurning(player) if player == Player::XPlayer => {
                    GameStatus::PlayerTurning(OPlayer)
                }
                GameStatus::PlayerTurning(_) => GameStatus::PlayerTurning(XPlayer),
            });
        }

        pub fn check_state(
            board: Board<u8, 9>,
            p: Player,
            coordinate: usize,
        ) -> Option<GameStatus> {
            let y = coordinate / LENGTH;
            let x = coordinate % LENGTH;

            let predicate = |mark: &Option<u8>| match p {
                Player::XPlayer => mark.is_some_with(|mark| Mark::from(*mark).is_x()),
                Player::OPlayer => mark.is_some_with(|mark| Mark::from(*mark).is_o()),
            };
            let vert = (0..LENGTH)
                .map(|x| y * LENGTH + x)
                .all(|idx| predicate(&board[idx]));
            let horizontal = (0..LENGTH)
                .map(|y| y * LENGTH + x)
                .all(|idx| predicate(&board[idx]));
            let main_diagonal = (0..LENGTH)
                .map(|i| i * LENGTH + i)
                .all(|idx| predicate(&board[idx]));
            let secondary_diagonal = (0..LENGTH)
                .map(|i| (LENGTH - i - 1) * LENGTH + i)
                .all(|idx| predicate(&board[idx]));
            let all_filled = board.iter().all(|m| m.is_some());
            if vert || horizontal || main_diagonal || secondary_diagonal {
                Some(GameStatus::PlayerWin(p))
            } else if all_filled {
                Some(GameStatus::Draw)
            } else {
                None
            }
        }
    }

    #[ink(event)]
    pub struct WaitingOpponent {
        #[ink(topic)]
        contract_account_id: AccountId,
        #[ink(topic)]
        player_x: AccountId,
        #[ink(topic)]
        player_0: Option<AccountId>,
    }

    #[ink(event)]
    pub struct GameStarted {
        #[ink(topic)]
        contract_account_id: AccountId,
        #[ink(topic)]
        player_x: AccountId,
        #[ink(topic)]
        player_o: AccountId,
    }

    #[ink(event)]
    pub struct PlayerTurn {
        #[ink(topic)]
        contract_account_id: AccountId,
        #[ink(topic)]
        turned_player: AccountId,
        #[ink(topic)]
        next_player: AccountId,
    }

    #[ink(event)]
    pub struct GameEnd {
        #[ink(topic)]
        winner: Option<AccountId>,
        #[ink(topic)]
        contract_account_id: AccountId,
    }
}
#[cfg(test)]
mod tests {
    use crate::board::{Board, Player};
    use crate::game_status::GameStatus;
    use crate::tic_tac_toe::TicTacToe;

    #[test]
    fn check_state() {
        let b = Board::<u8, 9>::new([
            Some(1),
            Some(1),
            Some(1),
            None,
            None,
            None,
            None,
            None,
            None,
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::OPlayer, 2).unwrap(),
            GameStatus::PlayerWin(Player::OPlayer)
        );

        let b = Board::<u8, 9>::new([
            Some(0),
            Some(0),
            Some(0),
            None,
            None,
            None,
            None,
            None,
            None,
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::XPlayer, 2).unwrap(),
            GameStatus::PlayerWin(Player::XPlayer)
        );

        let b = Board::<u8, 9>::new([
            None,
            Some(0),
            None,
            None,
            Some(0),
            None,
            None,
            Some(0),
            None,
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::XPlayer, 1).unwrap(),
            GameStatus::PlayerWin(Player::XPlayer)
        );

        let b = Board::<u8, 9>::new([
            None,
            None,
            Some(0),
            None,
            Some(0),
            None,
            Some(0),
            None,
            None,
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::XPlayer, 2).unwrap(),
            GameStatus::PlayerWin(Player::XPlayer)
        );

        let b = Board::<u8, 9>::new([
            Some(0),
            None,
            None,
            None,
            Some(0),
            None,
            None,
            None,
            Some(0),
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::XPlayer, 0).unwrap(),
            GameStatus::PlayerWin(Player::XPlayer)
        );

        let b = Board::<u8, 9>::new([
            Some(0),
            Some(1),
            Some(0),
            Some(0),
            Some(1),
            Some(0),
            Some(1),
            Some(0),
            Some(1),
        ]);
        assert_eq!(
            TicTacToe::check_state(b, Player::XPlayer, 0).unwrap(),
            GameStatus::Draw
        );
    }
}
