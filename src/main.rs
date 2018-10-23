extern crate irc;
extern crate rusqlite;
extern crate chrono;
extern crate regex;

use irc::client::prelude::*;
use std::env;
use rusqlite::Connection;
//use chrono::prelude::*;
use std::string::String;
use regex::Regex;

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

    conn.execute(
        "INSERT INTO command (name, text) VALUES (?1, ?2)",
        &[ &"repo".to_string(), &"https://github.com/YakPie/BotPie".to_string() ] 
    ).unwrap();

    conn.execute(
        "INSERT INTO command (name, text) VALUES (?1, ?2)",
        &[ &"schedule".to_string(), &"Check out schedule over at https://yakpie.com/".to_string() ] 
    ).unwrap();

    reactor.register_client_with_handler(client, move |client, orig_message| {
        print!("{}", orig_message);

		if let Command::PRIVMSG(channel, message) = orig_message.command {
            if message.starts_with("!help") {
                let mut stmt = conn.prepare("SELECT name FROM command").unwrap(); 
                let command_iter = stmt.query_map(&[], |row| {
                    let name : String = row.get(0);
                    return name;
                }).unwrap();

                let message = command_iter.fold(
                    String::from("Help command: help, addcommand, delcommand"),
                    |acc, command| {
                        let mut tmp_str = String::new();
                        tmp_str.push_str(&acc);
                        tmp_str.push_str(", ");
                        tmp_str.push_str(&command.unwrap());
                        tmp_str
                    }
                );
                
                client.send_privmsg(&channel, message).unwrap();
            }
            else if message.starts_with("!addcommand") {
                let re = Regex::new("^!addcommand ([a-zA-Z0-9]+) (.*)").unwrap();
                if let Some(cap) = re.captures_iter(&message).next() {
                    let command_name = cap[1].to_string();
                    let command_text = cap[2].to_string();

                    conn.execute(
                        "INSERT INTO command (name, text) VALUES (?1, ?2)",
                        &[&command_name, &command_text]
                    ).unwrap();

                    client.send_privmsg(&channel, "Command was added").unwrap();
                }
            }
            else if message.starts_with("!delcommand") {
                let re = Regex::new("^!delcommand ([a-zA-Z0-9]+)").unwrap();
                if let Some(cap) = re.captures_iter(&message).next() {
                    let command_name = cap[1].to_string();

                    conn.execute(
                        "DELETE FROM command WHERE name=?1",
                        &[&command_name]
                    ).unwrap();

                    client.send_privmsg(&channel, "Command was deleted").unwrap();
                }
            }
            else if message.starts_with("!") {
                let re = Regex::new("^!([a-zA-Z0-9]+)").unwrap();
                if let Some(cap) = re.captures_iter(&message).next() {
                    let command_name = cap[1].to_string();
                    let result : Result<String, rusqlite::Error> = conn.query_row(
                        "SELECT text FROM command WHERE name=?1",
                        &[ &command_name ],
                        |row| row.get(0)
                    );
                    match result {
                        Ok(command_text) => client.send_privmsg(&channel, command_text).unwrap(),
                        _ => ()
                    }
                }
            }
            
			if message.contains(client.current_nickname()) {
				client.send_privmsg(&channel, "beep boop").unwrap();
			}
		}
        Ok(())
    });

    reactor.run().unwrap();
}
