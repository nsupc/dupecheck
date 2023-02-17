use reqwest::Client;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::io;
use std::fs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="dupecheck")]
struct Opt {
    /// your main nation or ns email address
    #[structopt(name="user", short, long)]
    user_agent: Option<String>,
    /// the nation that you would like to check for duplicates
    #[structopt(short, long)]
    nation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Cards {
    #[serde(rename="DECK")]
    deck: Deck,
}

#[derive(Debug, Deserialize)]
struct Deck {
    #[serde(rename="CARD")]
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Card {
    #[serde(rename="CARDID")]
    id: u32,
    //#[serde(rename="CATEGORY")]
    //category: String,
    #[serde(rename="SEASON")]
    season: u8,
}

fn get_input(itype: &str) -> String {
    let mut buffer = String::new();

    let mut valid_input = false;

    while !valid_input {
        println!("Please enter your {}:", itype);

        if let Ok(_val) = io::stdin().read_line(&mut buffer) {
            buffer = buffer.trim().to_owned();

            if !buffer.is_empty() {
                valid_input = true;
            }
        }
    }

    buffer
}

async fn request(client: &Client, user: &str, nation: &str) -> Result<String, reqwest::Error> {
    let res = client
        .get(format!("https://www.nationstates.net/cgi-bin/api.cgi?q=cards+deck;nationname={}", nation))
        .header("User-Agent", format!("UPC's dupecheck, used by {}", user))
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

#[tokio::main]
async fn main() {
    let mut opt = Opt::from_args();

    if opt.user_agent.is_none() {
        opt.user_agent = Some(get_input("main nation"));
    }

    if opt.nation.is_none() {
        opt.nation = Some(get_input("target nation"));
    }

    let client = Client::new();

    let deck_response = request(&client, &opt.user_agent.unwrap(), &opt.nation.unwrap()).await;

    let cards: Cards = match deck_response {
        Ok(val) => from_str(&val).unwrap(),
        Err(e) => {
            println!("{:?}", e);
            return
        }
    };

    let mut deck: HashMap<String, u32> = HashMap::new();

    for card in cards.deck.cards {
        let url = format!("https://www.nationstates.net/page=deck/card={}/season={}", card.id, card.season);

        if deck.contains_key(&url) {
            deck.insert(url.clone(), deck[&url] + 1);
        } else {
            deck.insert(url.clone(), 1);
        }
    }

    let mut output = String::new();

    for (k, v) in deck.iter() {
        if *v != 1 {
            output.push_str(format!("{}: {}\n", k, v).as_str());
        }
    }

    if let Err(e) = fs::write("output.txt", &output) {
        println!("Error writing output to file: {}.\n Writing to terminal instead.\n\n{}", e, &output);
    } else {
        println!("Output written to output.txt");
    }

}

