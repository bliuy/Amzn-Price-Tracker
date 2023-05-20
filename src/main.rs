use crate::structs::AmznProductInformation;
use proto_messages::AmznScrapingRequest;
use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet};

mod functions;

#[tokio::main]
async fn main() {
    println!("Starting scraper service!");

    // Spwaning a separate listener thread - Responsible for listening on the MQTT broker.
    let mqtt_options: MqttOptions = MqttOptions::new("rust_price_scraper", "localhost", 1883); // Default options
    let (mqtt_client, mut mqtt_eventloop) = AsyncClient::new(mqtt_options, 100);
    match mqtt_client
        .subscribe("amzn_scraping_requests", rumqttc::QoS::AtLeastOnce)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    };

    // Polling the event loop
    loop {
        if let Ok(incoming_event) = mqtt_eventloop.poll().await {
            if let Event::Incoming(packet) = incoming_event {
                if let Packet::Publish(publish) = packet {
                    match publish.topic.as_str() {
                        "amzn_scraping_requests" => {
                            let payload_bytes = publish.payload.as_ref();
                            let decoded_request: AmznScrapingRequest = match prost::Message::decode(
                                payload_bytes,
                            ) {
                                Ok(i) => i,
                                Err(e) => {
                                    println!("Unable to decode message received in the 'amzn_scraping_request' topic. See error raised: {}", e);
                                    continue;
                                }
                            };
                            println!("{:?}", decoded_request);
                        }
                        unknown_topic => {
                            println!(
                                "Event with unknown topic received. See event topic: {}",
                                unknown_topic
                            );
                        }
                    }
                }
            }
        }
    }
}

mod proto_messages {
    include!(concat!(env!("OUT_DIR"), "/proto_messages.rs"));
}

async fn amzn_scraper(product: AmznProductInformation) -> () {
    todo!()
}

mod structs {

    pub struct AmznProductInformation {
        asin: String,
    }
}
