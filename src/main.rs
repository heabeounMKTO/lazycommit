mod structs;

use clap::Parser;
use reqwest;
use serde_json::{json, Value};
use std::process::Command;
use structs::{Changes, ChatCompletionResponse};

#[derive(Parser)]
struct CliArgs {}

fn get_diff() -> String {
    let mut git_diff = Command::new("git");
    git_diff.arg("diff");
    let diffc =
        String::from_utf8(git_diff.output().expect("[ERROR] CANNOT RUN GIT").stdout).unwrap();
    return diffc;
}

fn create_prompt(diff: String) -> Value {
    return json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "/nothink You are an expert in summarizing git diffs,
                            you are detail-oriented and your summarizations of the diffs are very concise and very easy to understand,
                            you can only reply in json below and nothing else!"
            },
            {
                "role": "user",
           "content": format!("{diff}\n{}", r#"
                    analyze this git diff above and respond in json format like below\n
                        {
                          "title": "summarized title",
                          "changes": {
                            "src/main.rs": ".."
                          }
                        }"#)
            }
        ],
        "max_tokens": -1
    });
}

fn format_commit(file: String, summary: String) -> String {
    return format!("- {} : {}\n", file, summary);
}

fn main() {
    let api_endpoint = "http://0.0.0.0:9999/v1/chat/completions";
    let api_key = "sk-my-penis";
    let diff = get_diff();
    let client = reqwest::blocking::Client::new();
    let payload = create_prompt(diff);
    let response = client
        .post(api_endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .unwrap();
    if response.status().is_success() {
        let chat_response: ChatCompletionResponse = response.json().unwrap();
        println!("CHAT RESP: {:?}", chat_response.choices[0].message.content);
        let mut clean_resp = chat_response.choices[0]
            .message
            .content
            .replace("<think>\n\n</think>\n\n", "");
        let parsed_resp: Changes = serde_json::from_str(&clean_resp).unwrap();
        let title = parsed_resp.title;
        let response = parsed_resp.changes;
        println!("commit title:\n{}\n", title);
        let mut commit_body: String = Default::default();
        for key in response.keys() {
            let _temp_commit =
                format_commit(key.to_owned(), response.get(key).unwrap().to_string());
            commit_body.push_str(&_temp_commit);
        }
        commit_body.push_str(&String::from("\nthis commit message was lazily generated with LLMs, please actually read the actual diffs."));
        let mut git_diff = Command::new("git");
        git_diff.arg("commit");
        git_diff.arg(format!("-m {}", title));
        git_diff.arg(format!("-m {}", commit_body));
        let commit_ = String::from_utf8(git_diff.output().expect("[ERROR] CANNOT RUN GIT").stdout).unwrap();
        println!("commit _res {:?}", commit_);
    } else {
        println!("[ERROR] http error, status not 200!");
    }
}
