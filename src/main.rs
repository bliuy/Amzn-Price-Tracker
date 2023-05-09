fn main() {
    println!("Hello, world!");
}

mod scraping_service {
    use std::{collections::HashMap, error};

    use reqwest::{Client, Response};
    use scraper::{error::SelectorErrorKind, Html, Selector};
    use thiserror::Error;

    struct ScrapingResult {
        name: String,
        price: i32,
        seller: String,
    }

    async fn make_request<'a, 'b>(
        client: &'b Client,
        amzn_asin: &'b str,
    ) -> Result<Response, ScrapingErrors<'a>> {
        // Creating the uri
        let uri = format!("https://www.amazon.sg/dp/{}", amzn_asin);

        // Issuing a GET request
        let response = client.get(uri).send().await?.error_for_status()?;
        Ok(response)
    }

    async fn parse_response<'a>(
        response: Response,
        selectors: &HashMap<String, Selector>,
    ) -> Result<(), ScrapingErrors<'a>> {
        // Getting the response content
        let response_content = response.text().await?;

        // Parsing the HTML content
        let document = Html::parse_document(&response_content);

        // Getting the product name
        const PRODUCT_TITLE_SELECTOR_STR: &str = "#productTitle";
        let product_name = css_selection(&document, PRODUCT_TITLE_SELECTOR_STR)
            .map_err(|e| ScrapingErrors::ScraperError(e.to_string()))?;

        let product_price_selector = scraper::Selector::parse(
            "span.a-price:nth-child(2) > span:nth-child(2) > span:nth-child(2)",
        )?;

        Ok(())
    }

    fn css_selection<'a>(
        document: &'a Html,
        selector_str: &'a str,
    ) -> Result<scraper::Node, ScrapingErrors<'a>> {
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
    enum ScrapingErrors<'a> {
        #[error("Error raised by the reqwest package. See error message: {0}")]
        ReqwestError(#[from] reqwest::Error),
        #[error("Error raised by the scraper package. See error message: {0}")]
        ScraperError(String),
        #[error("Error raised by the CSS Selector: See error message: {0}")]
        ScraperSelectorError(SelectorErrorKind<'a>),
        #[error("Error raised by the CSS selector: {selector_name}")]
        MissingCssSelectorElementError { selector_name: String },
        #[error("Error raised by the CSS selector: 'Value' field of selected element appears to be absent.")]
        MissingCssValue,
    }

    impl<'a> From<SelectorErrorKind<'a>> for ScrapingErrors<'a> {
        fn from(value: SelectorErrorKind<'a>) -> Self {
            ScrapingErrors::ScraperSelectorError(value)
        }
    }

    #[cfg(test)]
    mod tests {
        use reqwest::{Client, ClientBuilder};

        use super::{make_request, parse_response};

        // #[actix_web::test]
        // async fn test_make_request() {
        //     let client = ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/47.0.2526.111 Safari/537.36").build().unwrap();
        //     let asin = "B0BDHV5PX7";
        //     let response = make_request(&client, asin).await.unwrap();
        //     parse_response(response).await;
        // }
    }
}
