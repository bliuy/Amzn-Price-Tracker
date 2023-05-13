use std::{collections::HashMap, error, num::ParseIntError};

use reqwest::{Client, Response};
use scraper::{error::SelectorErrorKind, Html, Selector};
use thiserror::Error;

#[derive(Debug)]
struct ScrapingResult {
    name: String,
    price: i32,
    seller: String,
}

async fn make_request(client: &Client, amzn_asin: &str) -> Result<Response, ScrapingErrors> {
    // Creating the uri
    let uri = format!("https://www.amazon.sg/dp/{}", amzn_asin);

    // Issuing a GET request
    let response = client.get(uri).send().await?.error_for_status()?;
    Ok(response)
}

async fn parse_response(response: Response) -> Result<ScrapingResult, ScrapingErrors> {
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
enum ScrapingErrors {
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

    use super::{make_request, parse_response};

    #[tokio::test]
    async fn test_make_request() {
        let client = ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/47.0.2526.111 Safari/537.36").build().unwrap();
        let asin = "B0BDHV5PX7";
        let response = make_request(&client, asin).await.unwrap();
        let result = parse_response(response).await.unwrap();
        println!("{:#?}", result);
    }
}
