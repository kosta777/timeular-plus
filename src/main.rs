use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::prelude::*;
use std::io::BufReader;

type Error = Box<dyn std::error::Error>;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

const BASE_URL: &str = "https://api.timeular.com/api/v3";
const REPORT_FILE: &str = "./report.csv";
const API_KEY_FILE: &str = "./api.key";


fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String>
{
    let file = File::open(filename).expect("Failed to open file. No such file exists.");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[derive(Deserialize, Debug)]
struct MeResponse {
    data: Me,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Me {
    user_id: String,
    name: String,
    email: String,
    default_space_id: String,
}

async fn fetch_me(token: &str) -> Result<Me, Error> {
    let resp = CLIENT
        .get(&url("/me"))
        .header("Authorization", auth(token))
        .send()
        .await?
        .json::<MeResponse>()
        .await?;
    Ok(resp.data)
}

fn auth(token: &str) -> String { format!("Bearer {}", token) } 

#[derive(Deserialize, Debug)]
struct SpacesResponse {
    data: Vec<Space>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Space {
    id: String,
    name: String,
    default: bool,
    members: Vec<Member>,
    retired_members: Vec<RetiredMember>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Member {
    id: String,
    name: String,
    email: String,
    role: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RetiredMember {
    id: String,
    name: String,
}

async fn fetch_spaces(token: &str) -> Result<Vec<Space>, Error> {
    let resp = CLIENT
        .get(&url("/space"))
        .header("Authorization", auth(token))
        .send()
        .await?
        .json::<SpacesResponse>()
        .await?;
    Ok(resp.data)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ActivitiesResponse {
    activities: Vec<Activity>,
    inactive_activities: Vec<Activity>,
    archived_activities: Vec<Activity>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Activity {
    id: String,
    name: String,
    color: String,
    integration: String,
    space_id: String,
    device_side: Option<i64>,
}

async fn fetch_activities(token: &str) -> Result<Vec<Activity>, Error> {
    let resp = CLIENT
            .get(&url("/activities"))
            .header("Authorization", auth(token))
            .send()
            .await?
            .json::<ActivitiesResponse>()
            .await?;
    Ok(resp.activities)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SignInRequest {
    api_key: String,
    api_secret: String,
}

#[derive(Deserialize, Debug)]
struct SignInResponse {
    token: String,
}

fn url(path: &str) -> String {
    format!("{}{}", BASE_URL, path)
}

async fn sign_in(api_key: String, api_secret: String) -> Result<String, Error> {
    let body = SignInRequest {
            api_key,
            api_secret,
    };
    let resp = CLIENT
            .post(&url("/developer/sign-in"))
            .json(&body)
            .send()
            .await?
            .json::<SignInResponse>()
            .await?;
    Ok(resp.token)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let lines = lines_from_file(API_KEY_FILE);
    let api_key = lines[0][4..].to_string();
    let api_secret = lines[1][7..].to_string();

    println!("{}", api_key);
    println!("{}", api_secret);

    println!("signing in..");
    let token = sign_in(api_key, api_secret).await?;

    println!("fetching me and spaces...");
    let me = fetch_me(&token).await?;
    let spaces = fetch_spaces(&token).await?;
    println!("fetched spaces: {:?} for {:?}", spaces, me);

    Ok(())

}