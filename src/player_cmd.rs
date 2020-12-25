use mpris::{FindingError, PlayerFinder};
use std::convert::TryFrom;
use std::mem;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum PlayerCmd {
    Play,
    Pause,
    PlayPause,
}
pub const NUM_PLAYER_CMDS: u8 = PlayerCmd::PlayPause as u8 + 1;

pub fn perform(player_identity: &str, cmd: PlayerCmd) -> Result<(), FindingError> {
    // Initialise the dbus player finder.
    let player_finder = PlayerFinder::new()?;

    // Find the first player with the player identity asked for.
    let players = player_finder.find_all()?;
    let player = match players.iter().find(|p| p.identity() == player_identity) {
        Some(player) => player,
        None => return Err(FindingError::NoPlayerFound),
    };

    match cmd {
        PlayerCmd::Play => player.play(),
        PlayerCmd::Pause => player.pause(),
        PlayerCmd::PlayPause => player.play_pause(),
    }
    .map_err(|e| FindingError::DBusError(e))
}

impl TryFrom<u8> for PlayerCmd {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < NUM_PLAYER_CMDS {
            unsafe { mem::transmute(value) }
        } else {
            Err(())
        }
    }
}
