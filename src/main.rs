use anyhow::{Result};
use reqwest::{self};
use urlencoding::{encode};
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
    const MAX_LENGTH: usize = 100000;
    let mut words: Vec<Word> = vec![Word::new("披麻戴孝")];
    let mut remove: Vec<String> = vec![];
    let file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("idiom.json")?;
    let mut writer: LineWriter<File>= LineWriter::new(file);
    let file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.txt")?;
    let mut txt_writer: LineWriter<File>= LineWriter::new(file);
    loop {
        println!("loop...");
        println!("{}", words.len());
        println!("{}", remove.len());
        writeln!(&mut txt_writer, "{}", words.len())?;
        for word in words.iter().rev().take(10) {
            writeln!(&mut txt_writer, "{}", word.word)?;
        }
        write!(&mut txt_writer, "\n")?;
        let mut cache: Vec<Word> = vec![];
        for ct in 0..words.len() {
            let content: String = read_to_string("idiom.json")?;
            if words[ct].pinyin != "" || content.contains(&format!("\"word\":\"{}\"", &words[ct].word)) {
                remove.push(words[ct].word.clone());
                continue;
            }
            let url: String = format!("https://hanyuapp.baidu.com/dictapp/swan/termdetail?wd={}&client=pc&source_tag=2&lesson_from=xiaodu", encode(&words[ct].word));
            match reqwest::get(url).await {
                Ok(res) => {
                    let response: Value = res.json().await?;
                    let mut word_derivation: String = "".to_string();
                    let __: Option<()> = match response["data"]["chuChu"].as_array() {
                        Some(chu_chu) if !chu_chu.is_empty() => {
                            if let Some(item) = chu_chu[0].as_object() {
                                if item["citeOriginalText"].as_str().unwrap_or("") != "" {
                                    word_derivation = format!(
                                        "{} ——{}·{}·{}", 
                                        item["citeOriginalText"].as_str().unwrap_or(""),
                                        item["dynasty"].as_str().unwrap_or(""),
                                        item["author"].as_str().unwrap_or(""),
                                        item["source"].as_str().unwrap_or("")
                                    );
                                }
                            }
                            Some(())
                        },
                        Some(_) => None,
                        None => None
                    };
                    let mut word_explanation: String = "".to_string();
                    let __: Option<()> = match response.get("data").and_then(|data| data.get("definitionInfo")) {
                        Some(definition) => {
                            word_explanation = format!("{}", &definition["definition"]);
                            word_explanation = word_explanation.replace("\"", "");
                            Some(())
                        },
                        None => { None }
                    };
                    let mut word_pinyin: String = "".to_string();
                    let __: Option<()> = match response.get("data").and_then(|data| data.get("pinyin")) {
                        Some(pinyin) => {
                            word_pinyin = format!("{}", &pinyin);
                            word_pinyin = word_pinyin.replace("\"", "");
                            Some(())
                        },
                        None => { None }
                    };
                    if !content.contains(&words[ct].word) {
                        let now: DateTime<Local> = Local::now();
                        words[ct].pinyin = word_pinyin;
                        words[ct].explanation = word_explanation;
                        words[ct].derivation = word_derivation;
                        if words[ct].pinyin != "".to_string() || words[ct].explanation != "".to_string() {
                            println!("[{}]{} {} {} {}", now.format("%Y-%m-%d %H:%M:%S"), &words[ct].derivation, &words[ct].explanation, &words[ct].pinyin, &words[ct].word);
                            writeln!(&mut writer, "\t{},", serde_json::to_string(&words[ct])?)?;
                        }
                        remove.push(words[ct].word.clone());
                    }
                    let __: Option<()> = match response["data"]["relationInfo"]["relationList"].as_array() {
                        Some(relations) if !relations.is_empty() => {
                            for relation in relations {
                                if let Some(obj) = relation.as_object() && obj["name"].as_str().unwrap_or("") != "" {
                                    let text: &str= obj["name"].as_str().unwrap_or("");
                                    let content: String = read_to_string("idiom.json")?;
                                    if !content.contains(&format!("\"word\":\"{}\"", &text)) {
                                        cache.push(Word::new(&text));
                                    }
                                }
                            }
                            Some(())
                        },
                        Some(_) => None,
                        None => None
                    };
                    let __: Option<()> = match response["data"]["synonyms"].as_array() {
                        Some(synonyms) if !synonyms.is_empty() => {
                            for synonym in synonyms {
                                if let Some(obj) = synonym.as_object() && obj["name"].as_str().unwrap_or("") != "" {
                                    let text: &str= obj["name"].as_str().unwrap_or("");
                                    let content: String = read_to_string("idiom.json")?;
                                    if !content.contains(&format!("\"word\":\"{}\"", &text)) {
                                        cache.push(Word::new(&text));
                                    }
                                }
                            }
                            Some(())
                        },
                        Some(_) => None,
                        None => None
                    };
                    let __: Option<()> = match response["data"]["antonym"].as_array() {
                        Some(antonyms) if !antonyms.is_empty() => {
                            for antonym in antonyms {
                                if let Some(obj) = antonym.as_object() && obj["name"].as_str().unwrap_or("") != "" {
                                    let text: &str= obj["name"].as_str().unwrap_or("");
                                    let content: String = read_to_string("idiom.json")?;
                                    if !content.contains(&format!("\"word\":\"{}\"", &text)) {
                                        cache.push(Word::new(&text));
                                    }
                                }
                            }
                            Some(())
                        },
                        Some(_) => None,
                        None => None
                    };
                },
                Err(e) => {
                    println!("{}", e);
                    remove.push(words[ct].word.clone());
                },
            }
        }
        let content: String = read_to_string("idiom.json")?;
        words = cache;
        words.retain(|word: &Word| !remove.contains(&word.word) && !content.contains(&format!("\"word\":\"{}\"", &word.word)));
        if words.iter().all(|x: &Word| x.pinyin != "") || remove.len() > MAX_LENGTH {
            break;
        }
    }

    return Ok(())
}