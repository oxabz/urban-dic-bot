use lazy_static::lazy_static;
use regex::{Regex, Replacer};
use reqwest::Client;
use serde::Deserialize;
use urlencoding::encode;

lazy_static! {
    static ref CLIENT: Client = {
        Client::builder()
            .build()
            .expect("Error creating HTTP client")
    };
}

#[derive(Debug, Deserialize)]

struct UDResponse {
    list: Vec<Definition>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Definition {
    pub word: String,
    pub definition: String,
    pub example: String,
    pub thumbs_up: u32,
    pub thumbs_down: u32,
    pub permalink: String,
}

struct MDLinkReplacer;

impl Replacer for MDLinkReplacer {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let name = caps.name("name").unwrap().as_str();
        let url = format!(
            "[{name}](https://www.urbandictionary.com/define.php?term={})",
            encode(name)
        );
        dst.push_str(&url);
    }
}

impl Definition {
    pub fn md_formated_definition(&self) -> String {
        // Making the example a markdown citation
        let example = self.example.replace('\n', "\n> ");

        // Merging the definition and example
        let def = format!("{}\n\n> {}", self.definition, example);

        // Replacing the [] with the markdown links
        let re = Regex::new(r"\[(?P<name>[^\[\]]+)\]").unwrap();
        re.replace_all(def.as_str(), MDLinkReplacer).to_string()
    }

    pub fn word(&self) -> String {
        let mut chars = self.word.chars();
        chars
            .next()
            .map(|first_letter| first_letter.to_uppercase())
            .into_iter()
            .flatten()
            .chain(chars)
            .collect()
    }
}

pub async fn define(word: &str) -> Result<Vec<Definition>, reqwest::Error> {
    let url = format!("https://api.urbandictionary.com/v0/define?term={}", word);
    let resp = CLIENT.get(&url).send().await?;
    let json = resp.json::<UDResponse>().await?;
    Ok(json.list)
}
