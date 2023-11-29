#[macro_use]
extern crate rocket;

use rocket::{
    fairing::{Fairing, Info, Kind},
    fs::{relative, FileServer},
    http::Header,
    serde::{json::Json, Deserialize, Serialize},
    Request, Response,
};
use rusty_docs::{
    doc_gen::document_repository,
    models::{CodeFile, ItemKind, UserQuestionResponse},
    openai::ask_the_model,
    parsing::parse_repository,
    qdrant::embed_repository,
};
use std::path::{Path, PathBuf};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct AskReq<'a> {
    llm: &'a str,
    embedding_model: &'a str,
    openai_api_key: &'a str,
    qdrant_collection_name: &'a str,
    qdrant_url: &'a str,
    user_question: &'a str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DocumentReq<'a> {
    llm: &'a str,
    openai_api_key: &'a str,
    repository_path: &'a str,
    write_inside_repository: bool,
    write_to_json_path: &'a str,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct EmbedReq<'a> {
    embedding_model: &'a str,
    openai_api_key: &'a str,
    repository_path: &'a str,
    qdrant_collection_name: &'a str,
    qdrant_url: &'a str,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ParseReq<'a> {
    repository_path: &'a str,
    write_to_json_path: &'a str,
}

fn retrieve_code_files(repository_path: &str) -> Result<Vec<CodeFile<PathBuf>>, String> {
    if Path::new(repository_path)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("json"))
    {
        match std::fs::read_to_string(repository_path) {
            Ok(file_str) => match serde_json::from_str(&file_str) {
                Ok(code_files) => Ok(code_files),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        match parse_repository(repository_path, None) {
            Ok(code_files) => Ok(code_files),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {}

#[post("/ask", format = "application/json", data = "<req>")]
async fn ask<'a>(req: Json<AskReq<'a>>) -> Result<Json<UserQuestionResponse>, String> {
    match ask_the_model(
        req.llm,
        req.embedding_model,
        req.openai_api_key,
        req.qdrant_collection_name,
        req.qdrant_url,
        req.user_question,
    )
    .await
    {
        Ok(user_question_response) => Ok(Json(user_question_response)),
        Err(e) => Err(e.to_string()),
    }
}

#[post("/document", format = "application/json", data = "<req>")]
async fn document<'a>(req: Json<DocumentReq<'a>>) -> Result<String, String> {
    let code_files = retrieve_code_files(req.repository_path)?;

    if let Err(e) = document_repository(
        req.llm,
        code_files,
        &[ItemKind::All],
        req.openai_api_key,
        req.write_inside_repository,
        Some(req.write_to_json_path),
    )
    .await
    {
        Err(e.to_string())
    } else {
        Ok("Repository documented successfully.".into())
    }
}

#[post("/embed", format = "application/json", data = "<req>")]
async fn embed<'a>(req: Json<EmbedReq<'a>>) -> Result<String, String> {
    let code_files = retrieve_code_files(req.repository_path)?;

    if let Err(e) = embed_repository(
        code_files,
        req.embedding_model,
        req.openai_api_key,
        req.qdrant_collection_name,
        Some(20),
        req.qdrant_url,
    )
    .await
    {
        Err(e.to_string())
    } else {
        Ok("Repository embedded successfully.".into())
    }
}

#[post("/parse", format = "application/json", data = "<req>")]
fn parse(req: Json<ParseReq>) -> Result<String, String> {
    if let Err(e) = parse_repository(req.repository_path, Some(req.write_to_json_path)) {
        Err(e.to_string())
    } else {
        Ok("Repository parsed successfully.".into())
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", FileServer::from(relative!("/frontend/out")))
        .mount("/", routes![all_options])
        .mount("/", routes![ask])
        .mount("/", routes![document])
        .mount("/", routes![embed])
        .mount("/", routes![parse])
        .launch()
        .await?;

    Ok(())
}