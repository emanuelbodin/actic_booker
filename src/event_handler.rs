use dotenv::dotenv;
use lambda_runtime::{tracing, Error, LambdaEvent};
use std::{collections::HashMap, env};

#[derive(serde::Deserialize, Debug)]
pub struct Event {
    name: String,
    day: String,
    start_time: String,
}

pub(crate) async fn function_handler(event: LambdaEvent<Event>) -> Result<Vec<String>, Error> {
    dotenv().ok();
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);
    let username = env::var("USERNAME").unwrap();
    let password = env::var("PASSWORD").unwrap();
    let client = crate::actic::get_api_client(&username, &password)
        .await
        .unwrap();
    let all_classes = crate::actic::get_classes(&client).await.unwrap();
    let current_bookings = crate::actic::get_bookings(&client).await.unwrap();
    let matched_classes = crate::actic::get_matched_classes(
        &all_classes,
        &payload.name,
        &payload.day,
        &payload.start_time,
    );
    let booking_result = crate::actic::book_classes(&client, matched_classes, current_bookings)
        .await
        .unwrap();
    Ok(booking_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        let event = LambdaEvent::new(
            Event {
                name: String::from("test"),
                day: String::from("test"),
                start_time: String::from("test"),
            },
            Context::default(),
        );
        let response = function_handler(event).await.unwrap();
        assert_eq!(
            vec!["Successfully booked class: test @ test test"],
            response
        );
    }
}
