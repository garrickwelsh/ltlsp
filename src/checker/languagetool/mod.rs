use reqwest;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;
    use tokio;

    #[ignore]
    #[tokio::test]
    async fn query_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let mut form = HashMap::new();
        form.insert("language", "en-AU");
        form.insert(
            "text",
            "This is some somple test text. I'm hoping that language till tool will understand it.",
        );

        let res = client
            .post("http://localhost:8081/v2/check")
            .form(&form)
            .send()
            .await?;
        println!("{:?}", res);
        println!("{:?}", res.text().await?);
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool_with_serde() -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let mut request = HashMap::new();
        request.insert("language", "en-AU");
        println!("request created");

        let mut data = LanguageToolDocumentData {
            annotation: Vec::<LanguageToolText>::new(),
        };
        data.annotation.push(LanguageToolText::Markup("<h1>"));
        data.annotation.push(LanguageToolText::Text(
            "Here is som text that I'd like spell checked.",
        ));
        data.annotation.push(LanguageToolText::Text(
            "Is this something you're able to help me with?",
        ));
        data.annotation.push(LanguageToolText::Markup("</h1>"));

        let request_data = serde_json::to_string_pretty(&data).unwrap();
        request.insert("data", &request_data);

        println!("{}", &request_data);

        println!("request populated with data");
        let res = client
            .post("http://localhost:8081/v2/check")
            .form(&request)
            .send()
            .await;
        println!("{:?}", res);
        let res = res?;
        println!("{:?}", res);
        println!("{:?}", res.text().await?);
        Ok(())
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct LanguageToolDocumentData<'a> {
    annotation: Vec<LanguageToolText<'a>>,
}

#[derive(Serialize, Debug)]
pub(crate) enum LanguageToolText<'a> {
    #[serde(rename = "text")]
    Text(&'a str),
    #[serde(rename = "markup")]
    Markup(&'a str),
}

// #[derive(Serialize, Debug)]
// pub(crate) struct LanguageToolRequest<'a> {
//     language: &'a str,
//     data: LanguageToolDocumentData<'a>,
// }

// impl LanguageToolRequest<'_> {
//     fn new<'a>() -> LanguageToolRequest<'a> {
//         LanguageToolRequest {
//             language: "en-AU",
//             data: LanguageToolDocumentData {
//                 annotation: Vec::<LanguageToolText>::new(),
//             },
//         }
//     }
// }
