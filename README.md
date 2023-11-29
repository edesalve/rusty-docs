# rusty-docs

rusty-docs is a powerful tool designed to analyze, document, embed, and query Rust repositories. It offers the following features:

- **Parse**: Analyze and parse your Rust repository to gain insights into its structure and components. Obtain a structured JSON representation of the repository’s code elements.
- **Document**: Generate documentation for your Rust code **effortlessly**. Leverage rusty-docs’ documentation generation feature to create well-organized and detailed documentation for each code element. Seamlessly insert the produced documentation into the respective locations in your repository.
- **Embed**: Create embeddings from your repository code elements and store them on **Qdrant**, a powerful vector database. Configure Qdrant settings and choose your LLM preferences before generating embeddings.
- **Ask**: Open up a dialogue between you and **Jon** to gain insights, ask about functionalities, and explore the knowledge stored in the repository and the associated knowledge graph.

## Getting Started

To start using rusty-docs, follow these steps to build and run the application:

Navigate to the frontend directory and install the required npm packages:

```bash
cd frontend && npm install
```

Build the frontend:

```bash
npm run build
```

Navigate back to the root repository directory and build the Rust backend using Cargo:

```bash
cd .. && cargo build --release
```

Run the compiled Rust application:

```bash
./target/release/rusty-docs-app
```

Open [http://localhost:8000](~http://localhost:8000~) with your browser and start using rusty-docs!

## Examples

To make the following example work, your **Cargo.toml** should look like this:

```rust
[dependencies]
qdrant-client = "1.6"
rusty_docs = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Repo doc generation

The primary objective of this repository is to streamline the generation of high-quality documentation automatically. The initial step involves parsing the repository, wherein it is dissected into its fundamental components, known as **CodeElements**, utilizing the powerful syn library. CodeElements encapsulate diverse information from their corresponding code snippets, encompassing identifiers, types, implementors, dependencies, and more.
Subsequently, these CodeElements are meticulously fed into a finely tuned prompt that interfaces with the language model (LLM) responsible for generating the relevant documentation.

To optimize the documentation generation process, it is recommended to follow a two-step approach:

- **Generate Documentation for Basic Code Elements:**
  Begin by generating documentation for the simpler code elements such as functions, structs, and enums. Insert the generated documentation directly into the repository. This initial phase sets the groundwork for the subsequent step.
- **Generate Documentation for Modules and Impl Blocks:**
  Proceed to generate documentation for more complex structures like modules and impl blocks. By completing the documentation for these higher-level elements in the second phase, the llm gains a more comprehensive understanding, enhancing the quality of the generated documentation.

During this process, you have flexibility through the **write_inside_repository** and **write_to_json_path** options:

- If you choose to `write_inside_repository`, the generated documentation will be inserted directly into the appropriate location within the repository, positioned on top of the respective code element.
- Alternatively, selecting `write_to_json_path` generates a JSON file containing code elements along with the corresponding documentation.

⚠️ For operations involving `write_inside_repository`, it is advisable to perform the task in a new commit and carefully review the outcome to ensure accuracy and integrity. This meticulous approach ensures that the generated documentation seamlessly integrates with the existing codebase, fostering clarity and maintainability.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_files = rusty_docs::parsing::parse_repository("../repository_name/src", None)?;

    rusty_docs::doc_gen::document_repository(
        "gpt-4-1106-preview",
        code_files,
        &[rusty_docs::models::ItemKind::All],
        "openai_api_key",
        true,
        None::<&str>,
    )
    .await?;

    Ok(())
}
```

### Repo embeddings generation

Once your Qdrant server is deployed, and your repository is well documented, rusty-docs equips you with a comprehensive set of tools to effortlessly create embeddings and retrieve embedded elements.

⚠️ It’s important to note that the current implementation involves recreating collections for every new embedding procedure on existing collections. This process entails deleting the old collection and initiating the creation of a new one. Given the remarkably low cost of embedding models and the modest dimensions even in substantial repositories, this approach ensures efficiency and simplicity.
For users intending to experiment with different collections, it is essential to provide distinct collection names. This precautionary measure ensures a clear distinction between various test scenarios and prevents unintended overlap or interference between collections.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let qdrant_url = "url_to_your_qdrant_server";
    let client = qdrant_client::client::QdrantClient::from_url(qdrant_url).build()?;

    client.delete_collection("rustydocs_001").await?;
    rusty_docs::qdrant::new_collection(&client, "rustydocs_001").await?;

    let code_files = rusty_docs::parsing::parse_repository("../repository_name/src", None)?;

    embed_repository(
        code_files,
        "text-embedding-ada-002",
        "openai_api_key",
        "rustydocs_test_001",
        None,
        qdrant_url,
    )
    .await?;

    Ok(())
}
```

### Repo explorer

Once you’ve generated embeddings for your repository, delve into the world of insightful inquiry. rusty-docs opens the door for you to pose questions about your embedded repository elements. In the background, a sophisticated process unfolds as rusty-docs leverages the embeddings and constructs a finely-tuned, rich context with the elements retrieved from Qdrant. This meticulous approach ensures the delivery of the most accurate and relevant responses tailored to your queries.
In essence, rusty-docs becomes an ideal companion for uncovering insights about a new repository or facilitating the transfer of knowledge.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = rusty_docs::openai::ask_the_model(
        "gpt-4-1106-preview",
        "text-embedding-ada-002",
        "openai_api_key",
        "rustydocs_test_001",
        "url_to_your_qdrant_server",
        "How does repository_name works?"
    )
    .await?;

    eprintln!(
        "RESPONSE:\n{}\n\n SUGGESTED QUESTIONS:\n{:?}",
        response.response, response.suggested_questions,
    );

    Ok(())
}
```
