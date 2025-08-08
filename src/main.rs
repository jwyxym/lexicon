use anyhow::{Result};
use fantoccini::{
    Client, Locator,
    elements::{
        Element
    }
};
use urlencoding::{encode};
use std::{
    thread,
    time::Duration,
    fs::{
        File
    }
};
use serde::Serialize;

#[derive(Serialize)]
struct Word {
    derivation: String,//出处
    explanation: String,//解释
    pinyin: String,//拼音
    word: String
}

impl Word {
    pub fn new(word: &str) -> Self {
        Word {
            derivation: "".to_string(),
            explanation: "".to_string(),
            pinyin: "".to_string(),
            word: word.to_string()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    const MAX_LEN: usize = 20000;
    let client: Client = Client::new("http://localhost:9515").await?;
    let mut words: Vec<Word> = vec![Word::new("卷甲衔枚")];
    loop {
        print!("loop...\n");
        let mut remove: Vec<String> = vec![];
        for ct in 0..words.len() {
            if words[ct].pinyin != "" {
                continue;
            }
            let url: String = format!("https://hanyu.baidu.com/hanyu-page/term/detail?wd={}&ptype=zici", encode(&words[ct].word));
            client.goto(&url).await?;
            thread::sleep(Duration::from_millis(200));
            match client.find(Locator::Css(".idiom-container")).await {
                Ok(idiom_container) => {
                    let body: Element = client.find(Locator::Css(".term-container")).await?;
                    let pinyin_box: Element = body.find(Locator::Css(".pinyin-box")).await?;
                    let word_name: String = pinyin_box.find(Locator::Css(".name")).await?.text().await?;
                    let word_pinyin: String = pinyin_box.find(Locator::Css(".pinyin-item")).await?.text().await?.replace("[", "").replace("]", "");
                    let idioms:Vec<Element> = idiom_container.find_all(Locator::Css(".idiom-wrap")).await?;
                    let mut word_explanation: String = "".to_string();
                    let mut word_derivation: String = "".to_string();
                    for i in idioms.iter() {
                        let title: String = i.find(Locator::Css(".idiom-title")).await?.text().await?;
                        if word_explanation == "".to_string() && title.contains("基本释义") {
                            word_explanation = i.find(Locator::Css(".item-right")).await?.text().await?;
                        }
                        if word_derivation == "".to_string() && title.contains("出") && title.contains("处") {
                            word_derivation = format!("{} ——{}", i.find(Locator::Css(".item-text")).await?.text().await?, i.find(Locator::Css(".item-source")).await?.text().await?);
                        }
                    }
                    println!("{} {} {} {}", &word_derivation, &word_explanation, &word_pinyin, &word_name);
                    words[ct].pinyin = word_pinyin;
                    words[ct].explanation = word_explanation;
                    words[ct].derivation = word_derivation;
                    let relate_item: Vec<Element> = body.find_all(Locator::Css(".relate-item")).await?;
                    for i in relate_item.iter() {
                        let text: String = i.find(Locator::Css(".text")).await?.text().await?;
                        if words.iter().any(|x: &Word| x.word != text) && words.len() < MAX_LEN {
                            words.push(Word::new(&text));
                        }
                    }
                },
                Err(_) => {
                    remove.push(words[ct].word.clone());
                }
            }
        }
        words.retain(|word: &Word| !remove.contains(&word.word));
        if words.iter().all(|x: &Word| x.pinyin != "") {
            break;
        }
    }
    client.close().await?;
    let file = File::create("idiom.json")?;
    serde_json::to_writer_pretty(file, &words)?;
    return Ok(())
}