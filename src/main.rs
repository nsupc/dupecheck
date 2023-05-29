use reqwest::Client;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::io;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dupecheck")]
struct Opt {
    /// your main nation or ns email address
    #[structopt(name = "user", short, long)]
    user_agent: Option<String>,
    /// the nation that you would like to check for duplicates
    #[structopt(short, long)]
    nation: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Cards {
    #[serde(rename = "DECK")]
    deck: Deck,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Deck {
    #[serde(rename = "CARD")]
    cards: Option<Vec<Card>>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Card {
    #[serde(rename = "CARDID")]
    id: u32,
    #[serde(rename = "SEASON")]
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
        .get(format!(
            "https://www.nationstates.net/cgi-bin/api.cgi?q=cards+deck;nationname={}",
            nation
        ))
        .header("User-Agent", format!("UPC's dupecheck, used by {}", user))
        .send()
        .await?
        .text()
        .await?;

    Ok(res)
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let user_agent = match &opt.user_agent {
        Some(val) => val.to_string(),
        None => get_input("main nation"),
    };

    let target = match &opt.nation {
        Some(val) => val.to_string(),
        None => get_input("target nation"),
    };

    let client = Client::new();

    let deck_response = request(&client, &user_agent, &target).await;

    let cards: Cards = match deck_response {
        Ok(val) => match from_str(&val) {
            Ok(val) => val,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        },
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    let mut card_count: HashMap<String, u32> = HashMap::new();

    if let Some(deck) = cards.deck.cards {
        for card in deck {
            let url = format!(
                "https://www.nationstates.net/page=deck/card={}/season={}",
                card.id, card.season
            );

            card_count
                .entry(url)
                .and_modify(|val| *val += 1)
                .or_insert(1);
        }
    } else {
        println!("No cards found for {}", target);
        return;
    }

    let mut output: Vec<(String, u32)> = card_count
        .iter()
        .filter(|(_card, num)| **num > 1)
        .map(|(card, num)| (card.to_owned(), num.to_owned()))
        .collect();

    output.sort_by(|a, b| b.1.cmp(&a.1));

    for (card, count) in output {
        println!("{}: {}", card, count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_deck() {
        let test_xml = r#"
            <CARDS>
            <DECK>
            <CARD>
            <CARDID>2213620</CARDID>
            <CATEGORY>uncommon</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            <CARD>
            <CARDID>1723611</CARDID>
            <CATEGORY>common</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            <CARD>
            <CARDID>410</CARDID>
            <CATEGORY>legendary</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            <CARD>
            <CARDID>57712</CARDID>
            <CATEGORY>legendary</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            <CARD>
            <CARDID>1059603</CARDID>
            <CATEGORY>rare</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            <CARD>
            <CARDID>1528716</CARDID>
            <CATEGORY>uncommon</CATEGORY>
            <SEASON>1</SEASON>
            </CARD>
            </DECK>
            </CARDS>
            "#;

        let parsed: Cards = from_str(test_xml).unwrap();

        let expected = Cards {
            deck: Deck {
                cards: Some(vec![
                    Card {
                        id: 2213620,
                        season: 1,
                    },
                    Card {
                        id: 1723611,
                        season: 1,
                    },
                    Card { id: 410, season: 1 },
                    Card {
                        id: 57712,
                        season: 1,
                    },
                    Card {
                        id: 1059603,
                        season: 1,
                    },
                    Card {
                        id: 1528716,
                        season: 1,
                    },
                ]),
            },
        };

        assert_eq!(parsed, expected);
    }

    #[test]
    fn desearialize_empty_deck() {
        let test_xml = r#"
            <CARDS>
            <DECK/>
            </CARDS>
            "#;

        let parsed: Cards = from_str(test_xml).unwrap();

        let expected = Cards {
            deck: Deck { cards: None },
        };

        assert_eq!(parsed, expected);
    }
}
