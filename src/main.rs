use std::env;
mod models;
extern crate chrono;
extern crate reqwest;
use chrono::{DateTime, Utc};
use reqwest::redirect::Policy;
use models::UsersWorkoutsApiResponse;
use serde_json::json;

async fn fetch_session_auth_cookie(client: &reqwest::Client) -> Result<String, String> {
    let username = env::var("CROSSFIT_USERNAME").expect("CROSSFIT_USERNAME not set");
    let password = env::var("CROSSFIT_PASSWORD").expect("CROSSFIT_PASSWORD not set");

    // Define the login payload
    let params = [
        ("identity", username),
        ("credential", password),
        ("rememberme", "false".to_string()),
    ];

    // Make the POST request to login
    let res = client
        .post("https://torun.wod.guru/user/login")
        .form(&params)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    // Extract Set-Cookie header
    let headers = res.headers();
    if let Some(cookie_header) = headers.get("Set-Cookie") {
        let raw_cookie = cookie_header.to_str().unwrap();
        if raw_cookie.contains("PHPSESSID=") {
            let phpsessid = raw_cookie.split(';').nth(0).unwrap();
            let phpsession_value = phpsessid.split('=').nth(1).unwrap();
            return Ok(phpsession_value.to_string());
        }
    }

    Err("PHPSESSID cookie not found".to_string())
}

async fn fetch_user_workouts(
    client: &reqwest::Client,
    auth_cookie: String,
    time: DateTime<Utc>,
) -> Result<UsersWorkoutsApiResponse, String> {
    let formatted_date = time.format("%Y-%m-%d").to_string();
    let json_data = json!({
        "date": formatted_date,
        "v": time.timestamp()*100
    });

    let res = client
        .post("https://torun.wod.guru/my-training/user-workout/fetch-daily")
        .header("Cookie", format!("PHPSESSID={}", auth_cookie))
        .header("Content-Type", "application/json".to_string())
        .json(&json_data)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    let text = res.text().await.map_err(|err| err.to_string())?;

    let parsed_data: UsersWorkoutsApiResponse =
        serde_json::from_str(&text).map_err(|err| err.to_string())?;

    Ok(parsed_data)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .unwrap();

    match fetch_session_auth_cookie(&client).await {
        Ok(cookie_value) => {

    let now = Utc::now();
            match  fetch_user_workouts(&client, cookie_value, now).await {
                Ok(user_workouts) => {
                    for workout in user_workouts.userWorkouts.iter() {
                        if workout.classes.start_time == 405 {
                            println!("{:?} ({}) - {:#?}", workout.user.first_name, workout.user.id, workout.user.image);
                        }
                    }
                }
                Err(err) => println!("Error: {}", err),
            }
        }
        Err(err) => println!("Error: {}", err),
    }

    Ok(())
}
