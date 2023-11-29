use crate::{
    consts::{
        OPENAI_API_CHAT_COMPLETION_URL, OPENAI_API_EMBEDDING_URL, OPENAI_API_SEED,
        OPENAI_API_TOP_P, OPENAI_EMBEDDING_MODEL_MAX_TOKENS, SYSTEM_MSG_DOC_GENERATION,
        SYSTEM_MSG_USER_QUESTION,
    },
    models::{CodeElement, DocumentedCodeElement, ItemKind, UserQuestionResponse},
    qdrant::{retrieve_points_with_filter, retrieve_points_with_vector},
    utils::code_elment_from_scored_point,
};

use anyhow::{Error, Result};
use qdrant_client::prelude::*;
use serde_json::Value;
use tiktoken_rs::{get_bpe_from_model, get_completion_max_tokens};

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct ChatCompletionObject {
    id: Value,
    object: Value,
    created: Value,
    model: Value,
    system_fingerprint: Value,
    choices: Vec<Choice>,
    usage: Value,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Choice {
    index: Value,
    message: Message,
    finish_reason: Value,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: Value,
    object: Value,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: Value,
    object: Value,
    usage: Value,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Message {
    role: Value,
    content: String,
}

pub async fn ask_the_model(
    chat_model: &str,
    embedding_model: &str,
    openai_api_key: &str,
    qdrant_collection_name: &str,
    qdrant_url: &str,
    user_question: &str,
) -> Result<UserQuestionResponse> {
    let qdrant_client = QdrantClient::from_url(qdrant_url).build()?;

    let embedding = create_embedding(embedding_model, openai_api_key, user_question).await?;
    let retrieved_code_elements: Vec<CodeElement> =
        retrieve_points_with_vector(&qdrant_client, qdrant_collection_name, embedding)
            .await?
            .iter()
            .map(code_elment_from_scored_point)
            .collect();

    let mut system_msg = SYSTEM_MSG_USER_QUESTION.to_string();

    for code_element in retrieved_code_elements {
        expand_context(
            code_element,
            &qdrant_client,
            qdrant_collection_name,
            &mut system_msg,
        )
        .await;
    }

    if get_completion_max_tokens(chat_model, &format!("{system_msg}{user_question}"))? < 2000 {
        return Err(Error::msg(
            "The code snippet provided is too long: no room for model response",
        ));
    }

    let request_body = serde_json::json!({
        "model": chat_model,
        "messages": [
            {"role": "system", "content": system_msg},
            {"role": "user", "content": user_question}
        ],
        "response_format": {"type": "json_object"},
        "seed": OPENAI_API_SEED,
        "top_p": OPENAI_API_TOP_P,
    });

    // Make the API request
    let response = reqwest::Client::new()
        .post(OPENAI_API_CHAT_COMPLETION_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let response = response.json::<ChatCompletionObject>().await?;

        let user_question_response: UserQuestionResponse =
            serde_json::from_str(&response.choices.get(0).unwrap().message.content)?;

        Ok(user_question_response)
    } else {
        Err(Error::msg(format!(
            "Problems with response from OpenAI {chat_model}: {}",
            response.text().await?,
        )))
    }
}

pub fn count_tokens(model: &str, text: &str) -> u64 {
    let bpe = get_bpe_from_model(model).expect("Wrong model set");
    bpe.encode_with_special_tokens(text).len() as u64
}

pub async fn create_embedding(
    embedding_model: &str,
    openai_api_key: &str,
    text_to_embed: &str,
) -> Result<Vec<f32>> {
    if count_tokens(embedding_model, text_to_embed) > OPENAI_EMBEDDING_MODEL_MAX_TOKENS {
        return Err(Error::msg(
            "The code snippet provided is too long to be embedded",
        ));
    }

    let request_body = serde_json::json!({
        "input": text_to_embed,
        "model": embedding_model
    });

    let response = reqwest::Client::new()
        .post(OPENAI_API_EMBEDDING_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let response: EmbeddingResponse = response.json().await?;
        Ok(response.data.get(0).unwrap().embedding.clone())
    } else {
        Err(Error::msg(format!(
            "Problems with response from OpenAI {embedding_model}: {}",
            response.text().await?
        )))
    }
}

async fn expand_context(
    code_element: CodeElement,
    qdrant_client: &QdrantClient,
    qdrant_collection_name: &str,
    system_msg: &mut String,
) {
    *system_msg += &format!("\n{}\n", code_element.code);
    for code_element_id in code_element
        .dependencies
        .iter()
        .chain(&code_element.children)
    {
        let filter = qdrant_client::qdrant::Filter::must([
            qdrant_client::qdrant::Condition::has_id([code_element_id.get_hash()]),
        ]);

        if let Ok(retrieved_points) =
            retrieve_points_with_filter(qdrant_client, qdrant_collection_name, filter).await
        {
            if let Some(code) = retrieved_points
                .first()
                .and_then(|retrieved_point| retrieved_point.payload.get("code"))
            {
                *system_msg += &format!("\n{code}\n");
            }
        }
    }
}

pub async fn generate_documentation(
    chat_model: &str,
    openai_api_key: &str,
    ident: &str,
    kind: &ItemKind,
    location: &str,
    code: &str,
) -> Result<DocumentedCodeElement> {
    let user_msg = format!(
        "Provide the documentation to insert directly in the code of {ident}, a Rust {kind} whose location is {location}:
        
        {code}"
    );

    if get_completion_max_tokens(
        chat_model,
        &format!("{SYSTEM_MSG_DOC_GENERATION}{user_msg}"),
    )? < 2000
    {
        return Err(Error::msg(
            "The code snippet provided is too long: no room for model response",
        ));
    }

    let request_body = serde_json::json!({
        "model": chat_model,
        "messages": [
            {"role": "system", "content": SYSTEM_MSG_DOC_GENERATION},
            {"role": "user", "content": user_msg}
        ],
        "response_format": {"type": "json_object"},
        "seed": OPENAI_API_SEED,
        "top_p": OPENAI_API_TOP_P,
    });

    // Make the API request
    let response = reqwest::Client::new()
        .post(OPENAI_API_CHAT_COMPLETION_URL)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let response = response.json::<ChatCompletionObject>().await?;

        let mut raw_documented_code_element: DocumentedCodeElement =
            serde_json::from_str(&response.choices.get(0).unwrap().message.content)?;

        //TODO: improve this
        raw_documented_code_element.kind = raw_documented_code_element.kind.to_lowercase();

        raw_documented_code_element.location =
            raw_documented_code_element.location.replace(' ', "");

        raw_documented_code_element.location =
            raw_documented_code_element.location.replace("::", " :: ");

        Ok(raw_documented_code_element)
    } else {
        Err(Error::msg(format!(
            "Problems with response from OpenAI {chat_model}: {}",
            response.text().await?,
        )))
    }
}
