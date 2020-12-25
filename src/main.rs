#[macro_use]
extern crate log;

pub mod net;
pub mod player_cmd;

pub const DEFAULT_PORT: u16 = 1883;

use std::io::{self, Read};
use clap::{App, Arg};
use net::NetPlayerHandle;
use player_cmd::PlayerCmd;
use mpris::PlayerFinder;

fn main() {
    pretty_env_logger::init();

    let player_finder = PlayerFinder::new().expect("Unable to start player finder on DBUS");
    println!("Possible player identities: ");
    for player in player_finder.find_all().expect("Unable to list players") {
        println!("{}", player.identity());
    }

    let default_port_string = DEFAULT_PORT.to_string();
    let matches = App::new("Play it again, James")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(Arg::with_name("connect")
             .short("c")
             .value_name("SERVER_ADDRESS")
             .help("Sets the address of the mqtt server")
             .required(true))
        .arg(Arg::with_name("port")
             .short("p")
             .value_name("SERVER_PORT")
             .help("The port the server listens on")
            .default_value(&default_port_string))
        .arg(Arg::with_name("media_player")
             .short("m")
             .value_name("MEDIA_PLAYER")
             .help("Sets the player that should be synchronised")
             .required(true))
        .arg(Arg::with_name("name")
             .short("n")
             .value_name("CHANNEL_NAME")
             .help("Sets the name of the channel the player will listen/send to. Default is the same as `MEDIA_PLAYER`")
            )
        .get_matches();

    let player = matches
        .value_of("media_player")
        .expect("No player specified");
    let channel = matches.value_of("name").unwrap_or(player);

    let server_address = matches
        .value_of("connect")
        .expect("No server address specified");
    let server_port: u16 = match matches.value_of("port").expect("No port specified").parse() {
        Ok(port) => port,
        Err(err) => {
            error!("Not a valid port number: {:?}", err);
            warn!("Using default port {}", DEFAULT_PORT);
            DEFAULT_PORT
        }
    };

    let player_handle = NetPlayerHandle::start(server_address, server_port, player, channel);

    for c in io::stdin().lock().bytes() {
        let c = c.expect("Unable to read character from console");
        let cmd = match c as char {
            'p' => PlayerCmd::PlayPause,
            'r' => PlayerCmd::Play,
            'b' => PlayerCmd::Pause,
            '\n' => continue,
            _ => { error!("`{}` is not a player command. Please use [p]laypause, [r]esume and [b]reak", c); continue; }
        };

        player_handle.send_command(cmd);
    }

    player_handle.join();
}
