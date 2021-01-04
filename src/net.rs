use crate::player_cmd::{self, PlayerCmd};
use rumqttc::{Client, Connection, Event, Incoming, MqttOptions, Publish, QoS};
use std::convert::TryFrom;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use users::get_current_username;

pub struct NetPlayerHandle {
    outgoing_tx: Sender<PlayerCmd>,
    send_thread_handle: JoinHandle<()>,
    recv_thread_handle: JoinHandle<()>,
}

impl NetPlayerHandle {
    pub fn start(server_addr: &str, server_port: u16, name: &str, channel: &str) -> Self {
        info!("Connecting to server {}:{}", &server_addr, server_port);

        // Generate a semi-random username from the system username and a random number.
        let mut identity: String = get_current_username()
            .expect("Unable to get username, was the user deleted since starting the program?")
            .to_string_lossy()
            .into_owned();
        identity.push_str(&format!("{:08x}", rand::random::<u32>()));
        info!("Using identity {}", &identity);
        let mut mqtt_options = MqttOptions::new(&identity, server_addr, server_port);
        mqtt_options.set_keep_alive(5);

        let (mut client, connection) = Client::new(mqtt_options, 10);
        client
            .subscribe(channel, QoS::AtMostOnce)
            .expect("Unable to subscribe to player channel.");

        let (outgoing_tx, outgoing_rx) = mpsc::channel();
        let send_thread_handle = start_send_thread(client, channel, outgoing_rx);
        let recv_thread_handle = start_recv_thread(connection, channel, name);

        Self {
            outgoing_tx,
            send_thread_handle,
            recv_thread_handle,
        }
    }

    pub fn send_command(&self, cmd: PlayerCmd) {
        self.outgoing_tx
            .send(cmd)
            .expect("Unable to send command to command fifo");
    }

    pub fn join(self) {
        self.recv_thread_handle
            .join()
            .expect("Could not join net receiver thread");
        self.send_thread_handle
            .join()
            .expect("Could not join net sender thread");
    }
}

fn start_send_thread(
    mut client: Client,
    channel: &str,
    outgoing_rx: Receiver<PlayerCmd>,
) -> JoinHandle<()> {
    let channel = channel.to_owned();
    thread::spawn(move || {
        for cmd in outgoing_rx.iter() {
            match client.publish(&channel, QoS::AtLeastOnce, false, vec![cmd as u8]) {
                Ok(()) => info!("Published player command: {:?}", cmd),
                Err(e) => error!("Could not publish command: {:?}", e),
            }
        }
    })
}

fn start_recv_thread(mut connection: Connection, channel: &str, player: &str) -> JoinHandle<()> {
    let channel = channel.to_owned();
    let player = player.to_owned();
    thread::spawn(move || {
        for cmd in connection.iter() {
            let cmd = cmd.expect("Connection to server closed unexpectedly");
            info!("Received msg: {:?}", &cmd);

            if let Event::Incoming(Incoming::Publish(Publish { payload, topic, .. })) = cmd {
                if topic != channel {
                    continue;
                }

                match PlayerCmd::try_from(payload[0]) {
                    Ok(cmd) => {
                        println!("Player cmd received: {:?}", cmd);
                        match player_cmd::perform(&player, cmd) {
                            Ok(()) => (),
                            Err(e) => error!("Unable to perform command: {:}", e),
                        }
                    }
                    Err(()) => println!("Could not parse player command from: {:?}", payload),
                }
            }
        }
    })
}
