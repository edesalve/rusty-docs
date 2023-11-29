use crate::{
    models::{CodeElement, CodeFile, DocumentedCodeElement, FieldDescription, ItemKind},
    openai::generate_documentation,
    parsing::parse_file,
};

use anyhow::Result;

pub async fn document_repository<P: AsRef<std::path::Path>, W: AsRef<std::path::Path> + Clone>(
    chat_model: &str,
    code_files: Vec<CodeFile<P>>,
    kinds_to_document: &[ItemKind],
    openai_api_key: &str,
    write_inside_repository: bool,
    write_to_json_path: Option<W>,
) -> Result<()> {
    for code_file in code_files {
        crate::doc_gen::document_file(
            chat_model,
            &code_file.elements,
            code_file.path,
            kinds_to_document,
            openai_api_key,
            write_inside_repository,
            write_to_json_path.clone(),
        )
        .await?;
    }

    Ok(())
}

pub async fn document_file<P: AsRef<std::path::Path>, W: AsRef<std::path::Path> + Clone>(
    chat_model: &str,
    code_elements: &[CodeElement],
    file_to_document_path: P,
    kinds_to_document: &[ItemKind],
    openai_api_key: &str,
    write_inside_repository: bool,
    write_to_json_path: Option<W>,
) -> Result<()> {
    let mut raw_documented_code_elements = Vec::with_capacity(code_elements.len());

    for code_element in code_elements {
        let ident = code_element.code_element_id.ident.clone();
        let kind = code_element.code_element_id.kind.clone();
        let location = &code_element.code_element_id.location;
        let code = code_element.code.clone();

        //TODO: parallelize
        if kinds_to_document.contains(&kind) || kinds_to_document.contains(&ItemKind::All) {
            if let Ok(raw_documented_code_element) =
                generate_documentation(chat_model, openai_api_key, &ident, &kind, location, &code)
                    .await
            {
                raw_documented_code_elements.push(raw_documented_code_element);
            }
        }
    }

    if write_inside_repository {
        put_documentation_inside_repository(file_to_document_path, &raw_documented_code_elements)?;
    }
    if let Some(path) = write_to_json_path {
        write_documentation_to_file(path, &raw_documented_code_elements)?;
    }

    Ok(())
}

fn documentation_formatter(
    raw_element: &DocumentedCodeElement,
) -> (String, Option<Vec<FieldDescription>>) {
    let documentation = match raw_element.kind.as_str() {
        "fn" => {
            let doc_error_section = if raw_element.error_possible {
                format!("\n\n# Errors \n\n{}", raw_element.error_section)
            } else {
                String::new()
            };

            let doc_panic_section = if raw_element.panic_possible {
                format!("\n\n# Panics \n\n{}", raw_element.panic_section)
            } else {
                String::new()
            };

            let formatted_documentation = format!(
                "{}{}{} \n\n# Examples \n\n{}",
                raw_element.general_description,
                doc_error_section,
                doc_panic_section,
                raw_element.example_section
            );

            formatted_documentation
                .lines()
                .map(|line| format!("/// {line}"))
                .collect::<Vec<String>>()
                .join("\n")
        }
        "mod" => raw_element
            .general_description
            .lines()
            .map(|line| format!("//! {line}"))
            .collect::<Vec<String>>()
            .join("\n"),
        _ => raw_element
            .general_description
            .lines()
            .map(|line| format!("/// {line}"))
            .collect::<Vec<String>>()
            .join("\n"),
    };

    (
        documentation,
        raw_element.fields_or_variants_descriptions.clone(),
    )
}

fn find_start(
    code_elements: &[CodeElement],
    raw_documented_code_element: &DocumentedCodeElement,
) -> Option<usize> {
    if let Some(code_element) = code_elements.iter().find(|code_element| {
        code_element.code_element_id.ident == raw_documented_code_element.ident
            && code_element.code_element_id.kind.to_string() == raw_documented_code_element.kind
            && code_element.code_element_id.location == raw_documented_code_element.location
    }) {
        return Some(code_element.line_start);
    }

    None
}

pub(crate) fn pattern_formatter(ident: &str, kind: &ItemKind) -> String {
    match kind {
        ItemKind::Fn => format!("fn {ident}"),
        ItemKind::Const => format!("const {ident}"),
        ItemKind::Trait => format!("trait {ident}"),
        ItemKind::Type => format!("type {ident}"),
        ItemKind::Enum => format!("enum {ident}"),
        ItemKind::Struct => format!("struct {ident}"),
        _ => String::new(),
    }
}

pub fn put_documentation_inside_repository<P: AsRef<std::path::Path>>(
    file_to_document_path: P,
    raw_documented_code_elements: &[DocumentedCodeElement],
) -> Result<()> {
    let path = file_to_document_path.as_ref();

    if let Some(extension) = path.extension() {
        if extension == "rs" {
            // TODO: provide support for non-UTF-8 paths
            let mut location = path.to_str().unwrap().to_owned();
            if let Some(pos) = location.find("/src/") {
                location = location[pos + 5..].to_owned();
            }
            location = location
                .trim_end_matches(".rs")
                .replace('/', " :: ")
                .to_string();

            let file_raw_documented_code_elements: Vec<&DocumentedCodeElement> =
                raw_documented_code_elements
                    .iter()
                    .filter(|code_element| code_element.location.contains(&location))
                    .collect();

            for raw_documented_code_element in file_raw_documented_code_elements {
                // This operation is performed at each iteration to account for previous cycle modifications.
                let code_elements = parse_file(path)?.elements;
                let mut code_lines = read_file_to_document_lines(path)?;

                if let Some(line_start) = find_start(&code_elements, raw_documented_code_element) {
                    let formatted_documentation =
                        documentation_formatter(raw_documented_code_element);
                    code_lines.insert(line_start - 1, formatted_documentation.0);

                    write_lines_to_file(path, &code_lines)?;
                }
            }
        }
    }

    Ok(())
}

fn read_file_to_document_lines<P: AsRef<std::path::Path>>(file_path: P) -> Result<Vec<String>> {
    Ok(std::fs::read_to_string(file_path)?
        .lines()
        .map(String::from)
        .collect())
}

fn write_documentation_to_file<P: AsRef<std::path::Path>>(
    file_path: P,
    raw_documented_code_elements: &[DocumentedCodeElement],
) -> Result<()> {
    //TODO: check extension.
    let json_data = serde_json::to_string_pretty(&raw_documented_code_elements)?;
    Ok(std::fs::write(file_path, json_data)?)
}

fn write_lines_to_file<P: AsRef<std::path::Path>>(file_path: P, lines: &[String]) -> Result<()> {
    let modified_code = lines.join("\n");
    Ok(std::fs::write(file_path, modified_code)?)
}
