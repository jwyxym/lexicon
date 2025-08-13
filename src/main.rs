use anyhow::{Result};
use reqwest::{self};
use urlencoding::{encode};
use scraper::{Html, Selector};
use std::{
    fs::{
        File,
        OpenOptions,
        read_to_string
    },
    io::{
        Write,
        LineWriter,
    }
};
use serde::Serialize;
use serde_json::{Value};
use chrono::{Local, DateTime};

#[derive(Serialize)]
struct Word {
    derivation: String,//出处
    explanation: String,//解释
    pinyin: String,//拼音
    word: String
}

impl Word {
    pub fn new() -> Self {
        Word {
            derivation: "".to_string(),
            explanation: "".to_string(),
            pinyin: "".to_string(),
            word: "".to_string()
        }
    }
    pub fn name(&mut self, string: String) {
        self.word = string;
    }
    pub fn pinyin(&mut self, string: String) {
        self.pinyin = string;
    }
    pub fn derivation(&mut self, string: String) {
        self.derivation = string;
    }
    pub fn explanation(&mut self, string: String) {
        self.explanation = string;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("idiom.json")?;
    let mut writer: LineWriter<File>= LineWriter::new(file);
    for i in 30001..40000 {
        println!("{}", i);
        let url: String = format!("https://www.hanyuciku.com/cy/{}", i);
        match reqwest::get(url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(body) => {
                        let body: Html = Html::parse_document(&body);
                        let mut word: Word = Word::new();
                        let _ = match body.select(&Selector::parse("h1.has-text-danger").unwrap()).next() {
                            Some(element) => {
                                word.name(element.text().collect::<String>());
                                Some(())
                            },
                            None => None
                        };
                        let _ = match body.select(&Selector::parse(".ci-attrs").unwrap()).next() {
                            Some(element) => {
                                let _ = match element.select(&Selector::parse(".info-content").unwrap()).next() {
                                    Some(p) => {
                                        word.pinyin(p.text().collect::<String>());
                                        Some(())
                                    },
                                    None => None
                                };
                                Some(())
                            },
                            None => None
                        };
                        let _ = match body.select(&Selector::parse(".ext-item").unwrap()).next() {
                            Some(element) => {
                                let _ = match element.select(&Selector::parse(".info-content").unwrap()).next() {
                                    Some(p) => {
                                        word.derivation(p.text().collect::<String>());
                                        Some(())
                                    },
                                    None => None
                                };
                                Some(())
                            },
                            None => None
                        };
                        let _ = match body.select(&Selector::parse(".explain-box").unwrap()).next() {
                            Some(element) => {
                                word.explanation(element.text().collect::<String>().replace("\n\t\t复制\n\t\t", ""));
                                Some(())
                            },
                            None => None
                        };
                        let now: DateTime<Local> = Local::now();
                        if word.pinyin != "".to_string() && word.explanation != "".to_string() && word.word != "".to_string() && word.derivation != "".to_string() {
                            println!("[{}]{} {} {} {}", now.format("%Y-%m-%d %H:%M:%S"), &word.derivation, &word.explanation, &word.pinyin, &word.word);
                            writeln!(&mut writer, "\t{},", serde_json::to_string(&word)?)?;
                        }
                    },
                    Err(_) => {}
                }
            },
            Err(_) => {
                continue;
            }
        };
    }
    return Ok(())
}