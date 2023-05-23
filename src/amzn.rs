use std::{collections::HashMap, error, num::ParseIntError, ops::Div, time::Duration};

use reqwest::{Client, Response};
use scraper::{error::SelectorErrorKind, Html, Selector};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Debug)]
pub struct ScrapingResult {
    name: String,
    price: i32,
    seller: String,
}

pub async fn successful_scrape(scraping_result: &ScrapingResult) {
    // Printing the error message for now
    println!(
        "Scraping was successful! See details of product: {:?}.",
        scraping_result
    );
    // Placeholder for async delay
    sleep(Duration::from_secs(1)).await;
}

pub async fn unsuccessful_scrape(error_msg: &str) {
    // Printing the error message for now
    println!("Error message encountered: {error_msg}.");
    // Placeholder for async delay
    sleep(Duration::from_secs(1)).await;
}

pub async fn scrape(
    client: &Client,
    amzn_asin: &str,
    exponential_backoff_fn: Option<fn(u32) -> u32>,
) -> Result<Response, ScrapingErrors> {
    // Creating the uri
    let uri = format!("https://www.amazon.sg/dp/{}", amzn_asin);

    let retry_delay_func = match exponential_backoff_fn {
        Some(func) => func,
        None => |attempts: u32| {
            let result = (2_u32.pow(attempts) - 1).div(2);
            result
        },
    };

    let mut retry_attempts = 0;

    loop {
        let timeout = retry_delay_func(retry_attempts);
        sleep(Duration::from_secs(timeout.into())).await;
        match client.get(&uri).send().await?.error_for_status() {
            Ok(i) => return Ok(i),
            Err(e) if retry_attempts == 5 => return Err(e.into()),
            Err(_) => {
                retry_attempts += 1;
            }
        };
    }
}

pub async fn parse_response(response: Response) -> Result<ScrapingResult, ScrapingErrors> {
    // Getting the response content
    let response_content = response.text().await?;

    // Parsing the HTML content
    let document = Html::parse_document(&response_content);

    // Defining the required variables
    let name: String;
    let price: i32;
    let seller: String;

    // Getting the product name
    const PRODUCT_TITLE_SELECTOR_STR: &str = "#productTitle";
    match css_selection(&document, PRODUCT_TITLE_SELECTOR_STR)
        .map_err(|e| ScrapingErrors::ScraperError(e.to_string()))?
    {
        scraper::Node::Text(txt) => {
            name = txt.trim().to_string();
        }
        invalid_node => return Err(ScrapingErrors::InvalidCssValue),
    };

    // Getting the product price
    const PRODUCT_PRICE_SELECTOR_STR: &str =
        "span.a-price:nth-child(2) > span:nth-child(2) > span:nth-child(2)";
    match css_selection(&document, PRODUCT_PRICE_SELECTOR_STR)
        .map_err(|e| ScrapingErrors::ScraperError(e.to_string()))?
    {
        scraper::Node::Text(txt) => {
            price = txt.trim().to_string().replace(",", "").parse::<i32>()?;
        }
        invalid_node => return Err(ScrapingErrors::InvalidCssValue),
    };

    seller = "Amazon".to_string();

    // Creating the ScrapingResult
    let scraping_result = ScrapingResult {
        name,
        price,
        seller,
    };

    Ok(scraping_result)
}

fn css_selection(document: &Html, selector_str: &str) -> Result<scraper::Node, ScrapingErrors> {
    // Parsing the CSS Selector string
    let selector = scraper::Selector::parse(selector_str)?;

    // Selecting the relevant node
    let selected_result = document
        .select(&selector)
        .into_iter()
        .nth(0)
        .ok_or_else(|| ScrapingErrors::MissingCssSelectorElementError {
            selector_name: selector_str.to_owned(),
        })?
        .first_child()
        .ok_or_else(|| ScrapingErrors::MissingCssValue)?
        .value();

    Ok(selected_result.to_owned())
}

#[derive(Debug, Error)]
pub enum ScrapingErrors {
    #[error("Error raised by the reqwest package. See error message: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error raised by the scraper package. See error message: {0}")]
    ScraperError(String),
    #[error("Error raised by the CSS selector: {selector_name}")]
    MissingCssSelectorElementError { selector_name: String },
    #[error(
        "Error raised by the CSS selector: 'Value' field of selected element appears to be absent."
    )]
    MissingCssValue,
    #[error("Error raised by the CSS selector: 'Value' field of selected element appears to be of an incorrect type.")]
    InvalidCssValue,
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
}

impl<'a> From<SelectorErrorKind<'a>> for ScrapingErrors {
    fn from(value: SelectorErrorKind<'a>) -> Self {
        let error_msg = value.to_string();
        ScrapingErrors::ScraperError(error_msg)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::{Client, ClientBuilder};

    use super::{parse_response, scrape};

    // #[tokio::test]
    // async fn test_make_request() {
    //     let client = ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/47.0.2526.111 Safari/537.36").build().unwrap();
    //     let asin = "B0BDHV5PX7";
    //     let response = scrape(&client, asin).await.unwrap();
    //     let result = parse_response(response).await.unwrap();
    //     println!("{:#?}", result);
    // }
}
