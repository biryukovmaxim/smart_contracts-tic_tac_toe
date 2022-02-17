/// Errors that can occur upon calling this contract.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub enum Error {
    AnotherPlayerShouldTurn,
    CoordinateAlreadyFilled,
    CoordinateNotExists,
    GameNotStarted,
    GameAlreadyOver,
    ForGameNeedsAtLeast2Players,
    UnknownPlayer,
    WaitingAnotherDefinedPlayer,
    GameAlreadyStarted,
}
