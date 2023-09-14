use std::env;
extern crate reqwest;

async fn fetch_session_cookie() -> Result<String, String> {
    let client = reqwest::Client::new();

    let username = env::var("CROSSFIT_USERNAME").expect("WOD_GURU_USERNAME not set");
    let password = env::var("CROSSFIT_PASSWORD").expect("WOD_GURU_USERNAME not set");

    // Define the login payload
    let params = [("identity", username), ("credential", password), ("rememberme", "false".to_string())];

    // Make the POST request to login
    let res = client.post("https://torun.wod.guru/user/login")
        .form(&params)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    // Extract Set-Cookie header
    let headers = res.headers();
    if let Some(cookie_header) = headers.get("Set-Cookie") {
        let raw_cookie = cookie_header.to_str().unwrap();
        if raw_cookie.contains("PHPSESSID=") {
        let phpsessid =  raw_cookie.split(';').nth(0).unwrap();
            let phpsession_value = phpsessid.split('=').nth(1).unwrap();
            return Ok(phpsession_value.to_string());
        }
    }

    Err("PHPSESSID cookie not found".to_string())
}


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    match fetch_session_cookie().await {
        Ok(cookie_value) => println!("Got PHPSESSID cookie: {}", cookie_value),
        Err(err) => println!("Error: {}", err),
    }

    Ok(())
}
