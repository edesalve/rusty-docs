use crate::{
    doc_gen::pattern_formatter,
    models::{CodeElement, CodeElementID, CodeFile, ItemKind, SynItem},
    utils::{get_code_from_nested, get_item_from_nested, get_type_ident},
};

use anyhow::Result;
use syn::{parse_file as syn_parse_file, Item, __private::ToTokens, spanned::Spanned, Fields};

//TODO: manage partially qualified
fn contains_fully_qualified(ident: &str, location: &str, text: &str) -> bool {
    let fully_qualified: String = location
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        + "::"
        + ident;

    if text.contains(&fully_qualified) {
        return true;
    }

    false
}

//TODO: avoid to include occurrences inside strings.
fn contains_isolated(ident: &str, text: &str) -> bool {
    let re = regex::Regex::new(&format!(r"\b{ident}\b")).unwrap();
    re.is_match(text)
}

fn explode_use_tree(use_tree: &syn::UseTree, base: &str) -> Vec<(String, String)> {
    match use_tree {
        syn::UseTree::Path(use_path) => {
            let mut path = base.to_string();
            if !path.is_empty() {
                path.push_str(" :: ");
            }
            path.push_str(&use_path.ident.to_string());
            explode_use_tree(&use_path.tree, &path)
        }
        syn::UseTree::Name(use_name) => {
            let mut path = base.to_string();
            path.push_str(" :: ");
            path += &use_name.ident.to_string();

            vec![(path, use_name.ident.to_string())]
        }
        syn::UseTree::Rename(use_rename) => {
            let mut path = base.to_string();
            if !path.is_empty() {
                path.push_str(" :: ");
            }
            path.push_str(&use_rename.ident.to_string());
            vec![(
                format!("{} as {}", path, use_rename.rename),
                use_rename.rename.to_string(),
            )]
        }
        // TODO: understand how to manage this.
        syn::UseTree::Glob(_) => {
            vec![]
        }
        syn::UseTree::Group(use_group) => use_group
            .items
            .iter()
            .flat_map(|tree| explode_use_tree(tree, base))
            .collect(),
    }
}

pub(crate) fn extract_documentation(source: &str) -> String {
    let mut result = String::new();

    for line in source.lines() {
        if line.starts_with("//!") {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

// Dependencies and implementors are populated in a second step.
pub(crate) fn retrieve_code_element(
    item: syn::Item,
    code: &str,
    code_elements: &mut Vec<CodeElement>,
    location: &str,
    mut imports: Vec<(String, String)>,
) -> Option<CodeElementID> {
    let item = SynItem(item);
    let code_element_id =
        CodeElementID::new(item.get_ident(), item.get_kind(), location.to_string());

    if code_element_id.kind == ItemKind::Use {
        return None;
    }

    let children = match item.0 {
        Item::Fn(ref func) => {
            update_imports(&func.block, &mut imports);
            Vec::new()
        }
        //TODO: manage ident for trait implementation
        Item::Impl(ref impl_item) => {
            let impl_location = location.to_string()
                + " :: "
                + &format!(
                    "impl_{}",
                    get_type_ident(impl_item.self_ty.as_ref()).unwrap()
                );
            let mut children = Vec::new();

            for nested_item in &impl_item.items {
                let mut impl_imports = imports.clone();

                if let syn::ImplItem::Fn(func) = nested_item {
                    update_imports(&func.block, &mut impl_imports);
                }

                // I get the span before getting SynItem from nested_item.
                let nested_item_span = nested_item.span();
                let nested_item = get_item_from_nested(nested_item);

                let nested_code_element_id = CodeElementID::new(
                    nested_item.get_ident(),
                    nested_item.get_kind(),
                    impl_location.clone(),
                );

                // Location as the relative struct.
                let code_element = CodeElement {
                    code_element_id: nested_code_element_id.clone(),
                    code: get_code_from_nested(code, nested_item_span, Some(item.0.span())),
                    line_start: vec![nested_item_span.start().line],
                    imports: impl_imports.into_iter().map(|import| import.0).collect(),
                    children: Vec::new(),
                    dependencies: Vec::new(),
                    implementors: Vec::new(),
                };

                children.push(nested_code_element_id);

                code_elements.push(code_element);
            }

            children
        }
        Item::Mod(ref module_item) => {
            //TODO: improve this
            if module_item
                .ident
                .to_string()
                .to_lowercase()
                .contains("test")
            {
                return None;
            }

            let module_location = location.to_string() + " :: " + &module_item.ident.to_string();
            let mut children = Vec::new();

            if let Some(nested_items) = module_item.content.as_ref() {
                let module_imports = retrieve_imports(&nested_items.1);

                for nested_item in &nested_items.1 {
                    if let Some(nested_code_element_id) = retrieve_code_element(
                        nested_item.clone(),
                        &get_code_from_nested(code, nested_item.span(), Some(module_item.span())),
                        code_elements,
                        &module_location,
                        module_imports.clone(),
                    ) {
                        children.push(nested_code_element_id);
                    }
                }
            }

            children
        }
        Item::Trait(ref trait_item) => {
            let trait_location = location.to_string() + " :: " + &trait_item.ident.to_string();
            let mut children = Vec::new();

            for nested_item in &trait_item.items {
                // I get the span before getting SynItem from nested_item.
                let nested_item_span = nested_item.span();

                //TODO: manage verbatim kind

                let nested_item = get_item_from_nested(nested_item);
                let nested_code_element_id = CodeElementID::new(
                    nested_item.get_ident(),
                    nested_item.get_kind(),
                    trait_location.clone(),
                );

                // Location as the relative trait.
                let code_element = CodeElement {
                    code_element_id: nested_code_element_id.clone(),
                    code: get_code_from_nested(code, nested_item_span, Some(item.0.span())),
                    line_start: vec![nested_item_span.start().line],
                    imports: imports.iter().map(|import| import.0.clone()).collect(),
                    children: Vec::new(),
                    dependencies: Vec::new(),
                    implementors: Vec::new(),
                };

                children.push(nested_code_element_id);
                code_elements.push(code_element);
            }

            children
        }
        _ => Vec::new(),
    };

    let line_start =
        if code_element_id.kind == ItemKind::Struct || code_element_id.kind == ItemKind::Enum {
            structs_or_enums_localizer(&item)
        } else {
            vec![item.0.span().start().line]
        };

    let mut code_element = CodeElement {
        code_element_id: code_element_id.clone(),
        code: code.to_string(),
        line_start,
        imports: imports.into_iter().map(|import| import.0).collect(),
        children,
        dependencies: Vec::new(),
        implementors: Vec::new(),
    };

    // This is to differentiate between mods defined inside a mod.rs file from those defined inside actual code.
    if !(code_element_id.kind == ItemKind::Mod
        && item.0.span().start().line == item.0.span().end().line)
    {
        // Module level documentation needs to be placed at the first line of the module.
        if code_element_id.kind == ItemKind::Mod {
            *code_element
                .line_start
                .first_mut()
                .expect("There is always a line_start here") += 1;
        }

        code_elements.push(code_element);
    }

    Some(code_element_id)
}

fn structs_or_enums_localizer(item: &SynItem) -> Vec<usize> {
    let mut line_start = vec![item.0.span().start().line];
    match &item.0 {
        Item::Enum(enum_item) => {
            for variant in &enum_item.variants {
                line_start.push(variant.span().start().line);
            }
        }
        Item::Struct(struct_item) => {
            if let Fields::Named(fields) = &struct_item.fields {
                for field in &fields.named {
                    line_start.push(field.span().start().line);
                }
            }
        }
        _ => (),
    }

    line_start
}

pub fn parse_repository<P: AsRef<std::path::Path>>(
    repository_path: P,
    write_to_json_path: Option<P>,
) -> Result<Vec<CodeFile<std::path::PathBuf>>> {
    let mut parsed_repository = parsing_step_1(repository_path)?;
    parsed_repository = parsing_step_2(parsed_repository);

    //TODO: check file extension.
    if let Some(path) = write_to_json_path {
        let json_data = serde_json::to_string_pretty(&parsed_repository)?;
        std::fs::write(path, json_data)?;
    }

    Ok(parsed_repository)
}

pub fn parse_file<P: AsRef<std::path::Path>>(file_path: P) -> Result<CodeFile<P>> {
    let mut code_elements = Vec::new();
    let path = file_path.as_ref();

    if let Some(extension) = path.extension() {
        if extension == "rs" {
            let code = std::fs::read_to_string(path)?;
            let parsed = syn_parse_file(&code)?;

            // TODO: provide support for non-UTF-8 paths
            let mut location = path.to_str().unwrap().to_owned();
            if let Some(pos) = location.find("/src/") {
                location = location[pos + 5..].to_owned();
            }
            location = format!(
                "crate :: {}",
                location.trim_end_matches(".rs").replace('/', " :: ")
            );

            let imports: Vec<(String, String)> = retrieve_imports(&parsed.items);
            let mut children = Vec::new();

            for item in &parsed.items {
                let item_code = get_code_from_nested(&code, item.span(), None);

                if let Some(code_element_id) = retrieve_code_element(
                    item.clone(),
                    &item_code,
                    &mut code_elements,
                    &location,
                    imports.clone(),
                ) {
                    children.push(code_element_id);
                }
            }

            if let Some(stem) = path.file_stem() {
                if let Some(ident) = stem.to_str() {
                    let (ident, location) = if ident == "mod" {
                        let ident = path
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();

                        let location = location.replace(" :: mod", "");

                        (ident.to_string(), location)
                    } else {
                        (ident.to_string(), location)
                    };

                    code_elements.push(CodeElement {
                        code_element_id: CodeElementID::new(ident, ItemKind::Mod, location),
                        code,
                        line_start: vec![1],
                        imports: imports.into_iter().map(|import| import.0).collect(),
                        children,
                        dependencies: Vec::new(),
                        implementors: Vec::new(),
                    });
                }
            }
        }
    }

    Ok(CodeFile {
        path: file_path,
        elements: code_elements,
    })
}

fn retrieve_imports(items: &Vec<syn::Item>) -> Vec<(String, String)> {
    let mut imports = Vec::new();

    for item in items {
        if let syn::Item::Use(use_item) = item {
            imports.extend(explode_use_tree(&use_item.tree, ""));
        }
    }

    imports
}

fn parsing_step_1<P: AsRef<std::path::Path>>(
    directory_path: P,
) -> Result<Vec<CodeFile<std::path::PathBuf>>> {
    let mut code_files = Vec::new();
    let entries = std::fs::read_dir(directory_path)?;

    // First step initializing CodeElements
    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            code_files.extend(parsing_step_1(path)?);
        } else if path.is_file() {
            code_files.push(parse_file(path)?);
        }
    }
    Ok(code_files)
}

fn parsing_step_2(
    mut code_files: Vec<CodeFile<std::path::PathBuf>>,
) -> Vec<CodeFile<std::path::PathBuf>> {
    let mut total_code_elements: Vec<&mut CodeElement> = code_files
        .iter_mut()
        .flat_map(|code_file| &mut code_file.elements)
        .collect();

    add_implementors(&mut total_code_elements);
    add_dependencies(&mut total_code_elements);

    for code_element in total_code_elements {
        code_element.dependencies.sort();
        code_element.dependencies.dedup();

        code_element.implementors.sort();
        code_element.implementors.dedup();
    }

    code_files
}

fn add_dependencies(code_elements: &mut Vec<&mut CodeElement>) {
    let mut already_modified: Vec<&mut CodeElement> = Vec::with_capacity(code_elements.len());

    while let Some(code_element_to_modify) = code_elements.pop() {
        for code_element in code_elements.iter().chain(&already_modified) {
            if code_element
                .implementors
                .contains(&code_element_to_modify.code_element_id)
            {
                code_element_to_modify
                    .dependencies
                    .push(code_element.code_element_id.clone());
            }
        }

        already_modified.push(code_element_to_modify);
    }
}

//TODO: implement different logic for modules, impl blocks, traits.
fn add_implementors(code_elements: &mut Vec<&mut CodeElement>) {
    let mut already_modified: Vec<&mut CodeElement> = Vec::with_capacity(code_elements.len());

    while let Some(code_element_to_modify) = code_elements.pop() {
        // In implementors I avoid to insert these.
        if [ItemKind::Impl, ItemKind::Mod, ItemKind::Verbatim]
            .contains(&code_element_to_modify.code_element_id.kind)
        {
            already_modified.push(code_element_to_modify);
            continue;
        }

        // Searches for the ident of the CodeElement under investigation into all the
        // other CodeElements.
        for code_element in code_elements.iter().chain(&already_modified) {
            if [ItemKind::Impl, ItemKind::Mod, ItemKind::Verbatim]
                .contains(&code_element.code_element_id.kind)
            {
                continue;
            }

            //TODO: Manage plain comments
            // This way the search is performed only in the element body, not in the documentation.
            if let Some(index) = code_element.code.find(&pattern_formatter(
                &code_element.code_element_id.ident,
                &code_element.code_element_id.kind,
            )) {
                let (_, code_element_code) = code_element.code.split_at(index);

                // TODO: this must be improved.
                if (contains_isolated(
                    &code_element_to_modify.code_element_id.ident,
                    code_element_code,
                ) && verify_dependency(
                    &code_element_to_modify.code_element_id.location,
                    &code_element_to_modify.code_element_id.kind,
                    &code_element.code_element_id.location,
                    &code_element.imports,
                )) || contains_fully_qualified(
                    &code_element_to_modify.code_element_id.ident,
                    &code_element_to_modify.code_element_id.location,
                    code_element_code,
                ) {
                    code_element_to_modify
                        .implementors
                        .push(code_element.code_element_id.clone());
                }
            }
        }

        already_modified.push(code_element_to_modify);
    }

    *code_elements = already_modified;
}

//TODO: make it work for macros
fn update_imports(block: &syn::Block, surroundings_imports: &mut Vec<(String, String)>) {
    let mut scope_use_items = Vec::new();

    for stmt in &block.stmts {
        let stmt_string = stmt.to_token_stream().to_string();

        //TODO: understand if it's better to check for "use" or not.
        if stmt.to_token_stream().to_string().contains("use ") {
            if let Ok(item) = syn::parse_str::<syn::ItemUse>(&stmt_string) {
                scope_use_items.push(item);
            }
        }
    }

    let mut scope_imports: Vec<(String, String)> = scope_use_items
        .iter()
        .flat_map(|use_item| explode_use_tree(&use_item.tree, ""))
        .collect();

    for (import, import_name) in &mut *surroundings_imports {
        let index = if let Some((index, (scope_import, scope_import_name))) = scope_imports
            .iter()
            .enumerate()
            .find(|(_, (_, scope_import_name))| scope_import_name == import_name)
        {
            *import = scope_import.clone();
            *import_name = scope_import_name.clone();
            Some(index)
        } else {
            None
        };

        // Removes the overwritten import from scope imports.
        if let Some(index) = index {
            scope_imports.remove(index);
        }
    }
    // Pushes remaining scope imports.
    surroundings_imports.extend(scope_imports);
}

fn verify_dependency(
    analyzed_code_element_location: &str,
    analyzed_code_element_kind: &ItemKind,
    code_element_location: &str,
    code_element_imports: &[String],
) -> bool {
    if analyzed_code_element_location == "lib" {
        return true;
    }

    if analyzed_code_element_location == code_element_location {
        return true;
    }

    if analyzed_code_element_kind != &ItemKind::Mod
        && code_element_location.contains(analyzed_code_element_location)
    {
        return true;
    }

    // TODO: manage glob operator.
    code_element_imports
        .iter()
        .any(|import| import == analyzed_code_element_location)
}
