use mpris::{PlayerFinder, Player, FindingError, DBusError};

#[derive(Copy, Clone)]
pub enum PlayerCmd {
    Play,
    Pause,
    PlayPause
}

pub fn perform(player_identity: &str, cmd: PlayerCmd) -> Result<(), FindingError> {
    // Initialise the dbus player finder.
    let player_finder = PlayerFinder::new()?;

    // Find the first player with the player identity asked for.
    let players = player_finder.find_all()?;
    let player = match players.iter().find(|p| p.identity() == player_identity) {
        Some(player) => player,
        None => return Err(FindingError::NoPlayerFound)
    };

    match cmd {
        PlayerCmd::Play => player.play(),
        PlayerCmd::Pause => player.pause(),
        PlayerCmd::PlayPause => player.play_pause()
    }.map_err(|e| FindingError::DBusError(e))
}
