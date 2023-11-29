use crate::{
    consts::OPENAI_EMBEDDING_MAX_VECTOR_SIZE,
    models::{CodeElement, CodeFile},
    parsing::extract_documentation,
};

use anyhow::{Error, Result};
use futures::future::try_join_all;
#[allow(unused_imports)]
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    vectors_config::Config, Distance, Filter, RetrievedPoint, ScoredPoint, ScrollPoints,
    VectorParams, VectorsConfig,
};
use std::sync::Arc;

pub async fn embed_repository<P: AsRef<std::path::Path>>(
    code_files: Vec<CodeFile<P>>,
    embedding_model: &str,
    openai_api_key: &str,
    qdrant_collection_name: &str,
    qdrant_max_concurrent_tasks: Option<usize>,
    qdrant_url: &str,
) -> Result<()> {
    let code_elements = code_files
        .into_iter()
        .flat_map(|code_file| code_file.elements)
        .collect();

    upsert_code_element_embeddings(
        code_elements,
        qdrant_collection_name,
        embedding_model,
        qdrant_max_concurrent_tasks,
        qdrant_url,
        openai_api_key,
    )
    .await?;

    Ok(())
}

pub async fn new_collection(client: &QdrantClient, collection_name: &str) -> Result<bool> {
    Ok(client
        .create_collection(&CreateCollection {
            collection_name: collection_name.into(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: OPENAI_EMBEDDING_MAX_VECTOR_SIZE as u64,
                    distance: Distance::Dot as i32,
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })
        .await?
        .result)
}

pub async fn retrieve_points_with_filter(
    client: &QdrantClient,
    collection_name: &str,
    filter: Filter,
) -> Result<Vec<RetrievedPoint>> {
    Ok(client
        .scroll(&ScrollPoints {
            collection_name: collection_name.into(),
            filter: Some(filter),
            ..Default::default()
        })
        .await?
        .result)
}

pub async fn retrieve_points_with_vector(
    client: &QdrantClient,
    collection_name: &str,
    embedding: Vec<f32>,
) -> Result<Vec<ScoredPoint>> {
    Ok(client
        .search_points(&SearchPoints {
            collection_name: collection_name.into(),
            vector: embedding,
            limit: 3,
            with_payload: Some(true.into()),
            ..Default::default()
        })
        .await?
        .result)
}

pub async fn upsert_code_element_embeddings(
    code_elements: Vec<CodeElement>,
    collection_name: &str,
    embedding_model: &str,
    max_concurrent_tasks: Option<usize>,
    url: &str,
    openai_api_key: &str,
) -> Result<()> {
    let collection_name = Arc::new(collection_name.to_string());
    let embedding_model = Arc::new(embedding_model.to_string());
    let url = Arc::new(url.to_string());
    let openai_api_key = Arc::new(openai_api_key.to_string());

    match max_concurrent_tasks {
        Some(max_concurrent_tasks) => {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_tasks));
            let mut futures = Vec::with_capacity(max_concurrent_tasks);

            for code_element in code_elements {
                let permit = Arc::clone(&semaphore).acquire_owned().await?;
                let collection_name = Arc::clone(&collection_name);
                let embedding_model = Arc::clone(&embedding_model);
                let url = Arc::clone(&url);
                let openai_api_key = Arc::clone(&openai_api_key);

                futures.push(tokio::spawn(async move {
                    let _permit = permit; // This will automatically release the semaphore slot when the future completes
                    if let Ok(client) = QdrantClient::from_url(&url).build() {
                        let _res = upsert_embedding(
                            &client,
                            code_element,
                            &collection_name,
                            &embedding_model,
                            &openai_api_key,
                        )
                        .await;
                    }
                }));
            }

            let _res = try_join_all(futures).await?;
        }
        None => {
            if let Ok(client) = QdrantClient::from_url(&url).build() {
                for code_element in code_elements {
                    upsert_embedding(
                        &client,
                        code_element,
                        &collection_name,
                        &embedding_model,
                        &openai_api_key,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn upsert_embedding(
    client: &QdrantClient,
    mut code_element: CodeElement,
    collection_name: &str,
    embedding_model: &str,
    openai_api_key: &str,
) -> Result<()> {
    let embedding = if code_element.code_element_id.kind == crate::models::ItemKind::Mod {
        crate::openai::create_embedding(
            embedding_model,
            openai_api_key,
            &crate::parsing::extract_documentation(&code_element.code),
        )
        .await?
    } else {
        crate::openai::create_embedding(embedding_model, openai_api_key, &code_element.code).await?
    };

    if code_element.code_element_id.kind == crate::models::ItemKind::Mod {
        code_element.code = extract_documentation(&code_element.code);
        code_element.code = code_element.code.replace("//!", "");
    }

    code_element.code = code_element.code.replace("///", "");

    let Ok(payload) = serde_json::json!({
        "children": code_element.children,
        "code": code_element.code,
        "code_element_id": code_element.code_element_id,
        "dependencies": code_element.dependencies,
        "implementors": code_element.implementors,
        "imports": code_element.imports,
    })
    .try_into() else {
        return Err(Error::msg("Problems during Qdrant Payload generation"));
    };

    let points = vec![PointStruct::new(
        code_element.code_element_id.get_hash(),
        embedding,
        payload,
    )];
    client.upsert_points(collection_name, points, None).await?;

    Ok(())
}
