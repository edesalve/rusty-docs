use crate::utils::{deserialize_bool_from_str, get_type_ident};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use syn::Item;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CodeElement {
    pub code_element_id: CodeElementID,
    pub code: String,
    #[serde(skip)]
    pub line_start: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CodeElementID>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<CodeElementID>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub implementors: Vec<CodeElementID>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeElementID {
    pub ident: String,
    pub kind: ItemKind,
    pub location: String,
}

impl CodeElementID {
    pub(crate) fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn new(ident: String, kind: ItemKind, location: String) -> Self {
        Self {
            ident,
            kind,
            location,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeFile<P: AsRef<std::path::Path>> {
    pub path: P,
    pub elements: Vec<CodeElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentedCodeElement {
    pub ident: String,
    pub kind: String,
    pub location: String,
    pub general_description: String,
    #[serde(deserialize_with = "deserialize_bool_from_str")]
    pub panic_possible: bool,
    pub panic_section: String,
    #[serde(deserialize_with = "deserialize_bool_from_str")]
    pub error_possible: bool,
    pub error_section: String,
    pub example_section: String,
    #[serde(deserialize_with = "deserialize_bool_from_str")]
    pub has_fields_or_variants: bool,
    pub fields_or_variants_descriptions: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    All,
    Const,
    Enum,
    ExternCrate,
    Fn,
    ForeignMod,
    Impl,
    Macro,
    Mod,
    Static,
    Struct,
    Trait,
    TraitAlias,
    Type,
    Union,
    Use,
    Verbatim,
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ItemKind::All => "all",
            ItemKind::Const => "const",
            ItemKind::Enum => "enum",
            ItemKind::ExternCrate => "extern_crate",
            ItemKind::Fn => "fn",
            ItemKind::ForeignMod => "foreign_mod",
            ItemKind::Impl => "impl",
            ItemKind::Macro => "macro",
            ItemKind::Mod => "mod",
            ItemKind::Static => "static",
            ItemKind::Struct => "struct",
            ItemKind::Trait => "trait",
            ItemKind::TraitAlias => "trait_alias",
            ItemKind::Type => "type",
            ItemKind::Union => "union",
            ItemKind::Use => "use",
            ItemKind::Verbatim => "verbatim",
        };
        write!(f, "{name}")
    }
}

impl std::str::FromStr for ItemKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "all" => Ok(ItemKind::All),
            "const" => Ok(ItemKind::Const),
            "enum" => Ok(ItemKind::Enum),
            "externcrate" => Ok(ItemKind::ExternCrate),
            "fn" => Ok(ItemKind::Fn),
            "foreignmod" => Ok(ItemKind::ForeignMod),
            "impl" => Ok(ItemKind::Impl),
            "macro" => Ok(ItemKind::Macro),
            "mod" => Ok(ItemKind::Mod),
            "static" => Ok(ItemKind::Static),
            "struct" => Ok(ItemKind::Struct),
            "trait" => Ok(ItemKind::Trait),
            "traitalias" => Ok(ItemKind::TraitAlias),
            "type" => Ok(ItemKind::Type),
            "union" => Ok(ItemKind::Union),
            "use" => Ok(ItemKind::Use),
            "verbatim" => Ok(ItemKind::Verbatim),
            _ => Err(()),
        }
    }
}

impl From<&Item> for ItemKind {
    fn from(item: &Item) -> Self {
        match item {
            Item::Const(_) => ItemKind::Const,
            Item::Enum(_) => ItemKind::Enum,
            Item::ExternCrate(_) => ItemKind::ExternCrate,
            Item::Fn(_) => ItemKind::Fn,
            Item::ForeignMod(_) => ItemKind::ForeignMod,
            Item::Impl(_) => ItemKind::Impl,
            Item::Macro(_) => ItemKind::Macro,
            Item::Mod(_) => ItemKind::Mod,
            Item::Static(_) => ItemKind::Static,
            Item::Struct(_) => ItemKind::Struct,
            Item::Trait(_) => ItemKind::Trait,
            Item::TraitAlias(_) => ItemKind::TraitAlias,
            Item::Type(_) => ItemKind::Type,
            Item::Union(_) => ItemKind::Union,
            Item::Use(_) => ItemKind::Use,
            _ => ItemKind::Verbatim,
        }
    }
}

pub struct SynItem(pub Item);

impl SynItem {
    pub fn get_kind(&self) -> ItemKind {
        match self.0 {
            Item::Const(_) => ItemKind::Const,
            Item::Enum(_) => ItemKind::Enum,
            Item::ExternCrate(_) => ItemKind::ExternCrate,
            Item::Fn(_) => ItemKind::Fn,
            Item::ForeignMod(_) => ItemKind::ForeignMod,
            Item::Impl(_) => ItemKind::Impl,
            Item::Macro(_) => ItemKind::Macro,
            Item::Mod(_) => ItemKind::Mod,
            Item::Static(_) => ItemKind::Static,
            Item::Struct(_) => ItemKind::Struct,
            Item::Trait(_) => ItemKind::Trait,
            Item::TraitAlias(_) => ItemKind::TraitAlias,
            Item::Type(_) => ItemKind::Type,
            Item::Union(_) => ItemKind::Union,
            Item::Use(_) => ItemKind::Use,
            Item::Verbatim(_) => ItemKind::Verbatim,
            _ => unreachable!(),
        }
    }

    pub fn get_ident(&self) -> String {
        match &self.0 {
            Item::Const(item) => item.ident.to_string(),
            Item::Enum(item) => item.ident.to_string(),
            Item::ExternCrate(item) => item.ident.to_string(),
            Item::Fn(item) => item.sig.ident.to_string(),
            Item::ForeignMod(item) => String::default(), //TODO: manage this
            Item::Impl(item) => format!("impl_{}", get_type_ident(item.self_ty.as_ref()).unwrap()),
            Item::Macro(item) => {
                if let Some(ident) = &item.ident {
                    ident.to_string()
                } else {
                    String::default()
                }
            }
            Item::Mod(item) => item.ident.to_string(),
            Item::Static(item) => item.ident.to_string(),
            Item::Struct(item) => item.ident.to_string(),
            Item::Trait(item) => item.ident.to_string(),
            Item::TraitAlias(item) => item.ident.to_string(),
            Item::Type(item) => item.ident.to_string(),
            Item::Union(item) => item.ident.to_string(),
            Item::Verbatim(item) => item.to_string(), //TODO: manage this
            _ => String::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuestionResponse {
    pub response: String,
    pub suggested_questions: Vec<String>,
}
