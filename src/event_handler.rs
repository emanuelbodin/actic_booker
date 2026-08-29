use dotenv::dotenv;
use lambda_runtime::{tracing, Error, LambdaEvent};

#[derive(serde::Deserialize, Debug)]
pub struct Event {
    user: String,
    center_id: u32,
    name: String,
    day: String,
    start_time: String,
    latest: String,
}

pub(crate) async fn function_handler(event: LambdaEvent<Event>) -> Result<Vec<String>, Error> {
    dotenv().ok();
    let payload = event.payload;
    tracing::info!("Payload: {:?}", payload);

    let (username, password) = crate::credentials::get_credentials(&payload.user).await?;
    let client = crate::actic::get_api_client(&username, &password, payload.center_id)
        .await
        .map_err(|err| Error::from(err.to_string()))?;
    let all_classes = crate::actic::get_classes(&client)
        .await
        .map_err(|err| Error::from(err.to_string()))?;
    let current_bookings = crate::actic::get_bookings(&client)
        .await
        .map_err(|err| Error::from(err.to_string()))?;
    let matched_classes = crate::actic::get_matched_classes(
        &all_classes,
        &payload.name,
        &payload.day,
        &payload.start_time,
    );
    let only_book_latest = payload.latest == "true";
    let booking_result = crate::actic::book_classes(
        &client,
        matched_classes,
        current_bookings,
        only_book_latest,
    )
    .await
    .map_err(|err| Error::from(err.to_string()))?;

    Ok(booking_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    #[ignore = "requires Actic credentials and network access"]
    async fn test_event_handler() {
        let event = LambdaEvent::new(
            Event {
                user: String::from("kristian"),
                center_id: 123,
                name: String::from("test"),
                day: String::from("test"),
                start_time: String::from("test"),
                latest: String::from("test"),
            },
            Context::default(),
        );
        let response = function_handler(event).await.unwrap();
        assert!(!response.is_empty());
    }
}
