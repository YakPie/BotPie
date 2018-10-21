extern crate irc;
extern crate rusqlite;
extern crate chrono;

use irc::client::prelude::*;
use std::env;
use rusqlite::Connection;
use chrono::prelude::*;

fn main() {
    // IRC stup
    // TODO: refactor out into its own module
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

    // SQLite setup
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE command (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  text            TEXT NOT NULL
                  )", &[]).unwrap();  

    reactor.register_client_with_handler(client, move |client, message| {
        print!("{}", message);			

		if let Command::PRIVMSG(channel, message) = message.command {
            if message.contains("!help") {
                let mut stmt = conn.prepare("SELECT name FROM command").unwrap(); 
				client.send_privmsg(&channel, "Help commands:").unwrap();
                let command_iter = stmt.query_map(&[], |row| {
                    let name : String = row.get(0);
                    return name;
                }).unwrap();
                for command in command_iter {
				    client.send_privmsg(&channel, command.unwrap()).unwrap();
                }
            }
            else if message.contains("!addcommand") {
                let utc: DateTime<Utc> = Utc::now();
                conn.execute(
                    "INSERT INTO command (name, text) VALUES (?1, ?2)",
                    &[&"test".to_string(), &"test".to_string()]
                ).unwrap();
                print!("Tries to insert data into the database\n");
            }
            else if message.contains("!repo") {
				client.send_privmsg(&channel, "https://github.com/YakPie/BotPie").unwrap();
			}
            else if message.contains("!schedule") {
                client.send_privmsg(&channel, "Check out schedule over at https://yakpie.com/").unwrap();
			}
            else if message.contains("!") {
                // TODO: database lookup for command
            }
            
			if message.contains(client.current_nickname()) {
				client.send_privmsg(&channel, "beep boop").unwrap();
			}
		}
        Ok(())
    });

    reactor.run().unwrap();
}
