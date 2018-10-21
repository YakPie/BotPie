extern crate irc;

use irc::client::prelude::*;
use std::env;

fn main() {
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("yakpie".to_owned()),
        server: Some("irc.chat.twitch.tv".to_owned()),
        channels: Some(vec!["#yakpie".to_owned()]),
        password: Some(env::var("twitch_oath").unwrap()),
        ..Config::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, |client, message| {
        print!("{}", message);			

		if let Command::PRIVMSG(channel, message) = message.command {
			if message.contains("!repo") {
				client.send_privmsg(&channel, "https://github.com/YakPie/BotPie").unwrap();
			}

			if message.contains("!schedule") {
                client.send_privmsg(&channel, "Check out schedule over at https://yakpie.com/").unwrap();
			}	

			if message.contains(client.current_nickname()) {
				client.send_privmsg(&channel, "beep boop").unwrap();
			}
		}
        Ok(())
    });

    reactor.run().unwrap();
}
