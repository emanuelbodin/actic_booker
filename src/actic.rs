use chrono::{Datelike, NaiveDate};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, str};

pub struct ApiClient {
    pub client: reqwest::Client,
    pub user_id: String,
    pub center_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Activity {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Class {
    pub activity: Activity,
    pub date: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "bookingIdCompound")]
    pub booking_id_compound: String,
}

#[derive(Deserialize, Debug)]
pub struct ClassesData {
    pub classes: HashMap<String, Vec<Class>>, // Key is the date, value is a vector of classes
}

#[derive(Deserialize)]
pub struct Booking {
    activity: Activity,
    date: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
}

#[derive(Deserialize)]
pub struct BookingData {
    booking: Booking,
}

async fn get_auth(
    username: &str,
    password: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut body = HashMap::new();
    body.insert("password", &password);
    body.insert("username", &username);
    let data = client
        .post("https://webapi.actic.se/login")
        .json(&body)
        .send()
        .await?
        .text()
        .await?;
    let auth_data: Value = serde_json::from_str(&data)?;
    let access_token = auth_data["accessToken"].as_str().unwrap().to_string();
    let user_id = auth_data["person"]["personId"]["externalId"]
        .as_str()
        .unwrap()
        .to_string();
    return Ok((access_token, user_id));
}

pub async fn get_api_client(
    username: &str,
    password: &str,
    center_id: u32
) -> Result<ApiClient, Box<dyn std::error::Error>> {
    let (access_token, user_id) = get_auth(username, password).await.unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        "access-token",
        HeaderValue::from_str(&access_token).unwrap(),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    Ok(ApiClient {
        client: client,
        user_id: user_id,
        center_id: center_id.to_string(),
    })
}

pub async fn book_class(
    client: &ApiClient,
    booking_id_compound: String,
) -> Result<bool, reqwest::Error> {
    let url = format!(
        "https://webapi.actic.se/persons/{}/participations/{}",
        client.user_id, booking_id_compound
    );
    let res = client.client.post(url).send().await?;
    match res.status() {
        reqwest::StatusCode::OK => println!("Activity booked!"),
        _ => {
            println!("Failed to book activity!");
            println!("{}", res.text().await?);
            return Ok(false);
        }
    }
    Ok(true)
}

pub async fn get_classes(client: &ApiClient) -> Result<ClassesData, Box<dyn std::error::Error>> {
    let url = format!(
        "https://webapi.actic.se/persons/{}/centers/{}/classes",
        client.user_id,
        client.center_id
    );
    let res = client.client.get(url).send().await?.text().await?;
    let classes: ClassesData = serde_json::from_str(&res).unwrap();

    Ok(classes)
}

pub fn _print_classes(classes_data: &ClassesData) {
    for (date, classes) in &classes_data.classes {
        let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("Invalid date format");

        let day = dt.weekday().to_string();
        println!("{} {}", day, date);
        for class in classes {
            println!(
                "  activity: {}, compound_id: {} start: {}, end: {}",
                class.activity.name,
                class.booking_id_compound,
                class.start_time,
                class.end_time
            );
        }
    }
}

pub fn _print_bookings(booking_data: &HashMap<String, BookingData>) {
    for (booking_id, booking_data) in booking_data {
        println!("{}: {} @  {} {}-{}", booking_id, booking_data.booking.activity.name, booking_data.booking.date, booking_data.booking.start_time, booking_data.booking.end_time);
    }
}

pub async fn get_bookings(
    client: &ApiClient,
) -> Result<HashMap<String, BookingData>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://webapi.actic.se/persons/{}/participations/classes",
        client.user_id
    );
    let res: String = client.client.get(url).send().await?.text().await?;
    let booked_classes: HashMap<String, BookingData> = serde_json::from_str(&res).unwrap_or_else(|_| {
        HashMap::new()
    });
    return Ok(booked_classes);
}

pub fn get_matched_classes(
    classes_data: &ClassesData,
    name: &str,
    day: &str,
    time: &str,
) -> Vec<Class> {
    let mut filtered_classes: Vec<Class> = Vec::new();
    for (date, classes) in &classes_data.classes {
        let dt = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("Invalid date format");
        let week_day = dt.weekday().to_string();
        if day == week_day {
            for class in classes {
                if class.activity.name == name && class.start_time == time {
                    filtered_classes.push(class.clone());
                }
            }
        }
    }
    return filtered_classes;
}

pub async fn book_classes(
    client: &ApiClient,
    matched_classes: Vec<Class>,
    current_bookings: HashMap<String, BookingData>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut booking_result: Vec<String> = vec![];
    for class in matched_classes {
        if current_bookings.contains_key(&class.booking_id_compound)
            && current_bookings[&class.booking_id_compound]
                .booking
                .activity
                .name
                == class.activity.name
            && current_bookings[&class.booking_id_compound].booking.date == class.date
        {
            booking_result.push(format!(
                "Already booked class: {} @ {} {}",
                class.activity.name, class.date, class.start_time
            ));
            continue;
        }
        println!("Booking class: {}", class.booking_id_compound);
        let booking_successful = book_class(&client, class.booking_id_compound).await?;
        if booking_successful {
            booking_result.push(format!(
                "Successfully booked class: {} @ {} {}",
                class.activity.name, class.date, class.start_time
            ));
        } else {
            booking_result.push(format!(
                "Failed to book class: {} @ {} {}",
                class.activity.name, class.date, class.start_time
            ));
        }
    }
    Ok(booking_result)
}
