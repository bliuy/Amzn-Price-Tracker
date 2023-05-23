use crate::structs::AmznProductInformation;
use actix_web::{guard, web, App, HttpServer};
use proto_messages::AmznScrapingRequest;
use reqwest::ClientBuilder;
use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet};

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
                web::resource("/hello_world")
                    .route(web::route().guard(guard::Get()).to(services::hello_world)),
            )
            .service(
                web::resource("/amzn_request")
                    .route(web::route().guard(guard::Post()).to(services::amzn_request)),
            )
    })
    .bind(("localhost", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}

mod services {
    use std::time::Duration;

    use actix_web::{
        rt::spawn,
        web::{self, Bytes},
        HttpRequest, HttpResponse, Responder, ResponseError, Result,
    };
    use futures::future;
    use reqwest::Client;
    use tokio::time::sleep;

    use crate::{
        amzn::{parse_response, scrape, successful_scrape, unsuccessful_scrape},
        proto_messages::AmznScrapingRequest,
        structs::AmznProductInformation,
    };

    pub async fn hello_world() -> impl Responder {
        HttpResponse::Ok().body("Hello World!")
    }

    pub async fn amzn_request(
        reqwest_client: web::Data<Client>,
        request: HttpRequest,
        request_payload: web::Bytes,
    ) -> Result<String> {
        let amzn_scraping_request: AmznScrapingRequest = prost::Message::decode(request_payload)
            .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

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
                return Ok(format!("Status: {}/{} requests were successfully scraped.", successful, total))
            },
            Err(e) => {
                return Err(actix_web::error::ErrorBadRequest(e))
            },
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
