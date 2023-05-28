use crate::structs::AmznProductInformation;
use actix_web::{guard, web, App, HttpServer};
use proto_messages::AmznScrapingRequest;
use reqwest::ClientBuilder;

mod amzn;

#[actix_web::main]
async fn main() {
    println!("Scraper service starting");

    // Constructing the reqwest Client - Used for scraping
    static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/47.0.2526.111 Safari/537.36";

    // Spawning a new HttpServer
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(
                ClientBuilder::new().user_agent(USER_AGENT).build().unwrap(),
            ))
            .service(
                web::resource("/hello_world").route(
                    web::route()
                        .guard(guard::Any(guard::Get()).or(guard::Post())) // Allowing POST - Used for PubSub.
                        .to(services::hello_world),
                ),
            )
            .service(
                web::resource("/amzn_request")
                    .route(web::route().guard(guard::Post()).to(services::amzn_request)),
            )
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}

mod services {
    use std::time::Duration;

    use actix_web::{
        rt::spawn,
        web::{self, Bytes, Json},
        HttpRequest, HttpResponse, Responder, ResponseError, Result,
    };
    use base64::{engine::general_purpose, Engine};
    use futures::future;
    use reqwest::Client;
    use tokio::time::sleep;

    use crate::{
        amzn::{parse_response, scrape, successful_scrape, unsuccessful_scrape},
        proto_messages::AmznScrapingRequest,
        pubsub::PubSubMessage,
        structs::AmznProductInformation,
    };

    pub async fn hello_world() -> impl Responder {
        HttpResponse::Ok().body("Hello World!")
    }

    pub async fn amzn_request(
        reqwest_client: web::Data<Client>,
        request_payload: Json<PubSubMessage>,
    ) -> Result<String> {
        // Printing out the Json Payload for debugging purposes
        println!("{:#?}", request_payload);

        let encoded_payload = request_payload.message.data.as_bytes();
        let amzn_scraping_request: AmznScrapingRequest = match general_purpose::STANDARD
            .decode(encoded_payload)
        {
            Ok(i) => {
                match prost::Message::decode(i.as_ref()) {
                    Ok(i) => i,
                    Err(e) => {
                        println!(
                            "Unable to decode message into intended class. See error: {}",
                            e
                        );
                        return Ok("Message was received but was unable to be decoded.".to_string());
                        // Incorrect schema detected; no point in retrying.
                    }
                }
            }
            Err(e) => {
                println!("Base64 decoding failed. See error: {}", e);
                return Ok("Message was received but was unable to be decoded.".to_string());
                // Incorrect schema detected; no point in retrying.
            }
        };

        let futures = amzn_scraping_request
            .product_codes
            .into_iter()
            .map(|asin| {
                let client = reqwest_client.get_ref().clone();
                spawn(async move {
                    match scrape(&client, &asin, None).await {
                        Ok(res) => match parse_response(res).await {
                            Ok(result) => {
                                successful_scrape(&result).await;
                                return 1;
                            }
                            Err(e) => {
                                unsuccessful_scrape(&e.to_string()).await;
                                return 0;
                            }
                        },
                        Err(e) => {
                            unsuccessful_scrape(&e.to_string()).await;
                            return 0;
                        }
                    };
                })
            })
            .collect::<Vec<_>>();

        match future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<u32>, _>>()
        {
            Ok(i) => {
                let successful: u32 = i.iter().sum();
                let total = i.len();
                return Ok(format!(
                    "Status: {}/{} requests were successfully scraped.",
                    successful, total
                ));
            }
            Err(e) => return Err(actix_web::error::ErrorBadRequest(e)),
        };
    }
}

mod proto_messages {
    include!(concat!(env!("OUT_DIR"), "/proto_messages.rs"));
}

async fn amzn_scraper(product: AmznProductInformation) -> () {
    todo!()
}

mod structs;

mod pubsub {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PubSubMessage {
        pub(crate) message: PubSubMessageMessage,
        pub(crate) subscription: String,
    }
    #[derive(Debug, Deserialize, Serialize)]
    pub struct PubSubMessageMessage {
        pub(crate) attributes: HashMap<String, String>,
        pub(crate) data: String,         // Base64 encoded
        pub(crate) messageId: String,    // Appears to be identical to message_id
        pub(crate) message_id: String,   // String of numbers
        pub(crate) publishTime: String,  // Appears to be identical to publish_time
        pub(crate) publish_time: String, // Timestamp when the message was published. In the following format: YYYY-MM-DDTHH:MM:SS.sssZ (Example: 2021-02-26T19:13:55.749Z)
    }
}
