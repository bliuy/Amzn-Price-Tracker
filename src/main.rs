use crate::structs::AmznProductInformation;
use rumqttc::{AsyncClient, Client, Event, MqttOptions, Packet};

mod functions;

#[tokio::main]
async fn main() {
    println!("Starting scraper service!");

    // Spwaning a separate listener thread - Responsible for listening on the MQTT broker.
    let mqtt_options: MqttOptions = MqttOptions::new("rust_price_scraper", "localhost", 1883); // Default options
    let (mqtt_client, mut mqtt_eventloop) = AsyncClient::new(mqtt_options, 100);
    match mqtt_client
        .subscribe("price_scraper_topic", rumqttc::QoS::AtLeastOnce)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    };

    // Polling the event loop
    loop {
        if let Ok(incoming_mqtt_event) = mqtt_eventloop.poll().await {
            if let Event::Incoming(packet) = incoming_mqtt_event {
                if let Packet::Publish(publish) = packet {
                    let payload_string = std::str::from_utf8(&publish.payload).unwrap();
                    println!("{}", payload_string);
                }
            }
        };
        // match mqtt_eventloop.poll().await {
        // Ok(incoming_mqtt_event) => match incoming_mqtt_event {
        //     rumqttc::Event::Incoming(packet) => match packet {
        //         rumqttc::Packet::Publish(publish) => {
        //             publish.
        //         },
        //         _ => {}
        //     },
        //     rumqttc::Event::Outgoing(packet) => {
        //         println!("Outgoing: {:#?}", packet);
        //     }
        // },
        // Err(_) => {}
        // }
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
