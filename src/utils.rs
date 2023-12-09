use crate::models::{CodeElement, SynItem};

use proc_macro2::Span;
use qdrant_client::qdrant::ScoredPoint;
use serde_json::{from_value, Map, Value, Value::Object};
use syn::{__private::ToTokens, parse_str};

pub(crate) fn code_elment_from_scored_point(scored_point: &ScoredPoint) -> CodeElement {
    let json_map: Map<String, Value> = scored_point
        .payload
        .iter()
        .map(|(key, value)| (key.clone(), value.clone().into_json()))
        .collect();

    from_value(Object(json_map)).expect("msg")
}

pub(crate) fn deserialize_bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(s),
            &"true or false",
        )),
    }
}

pub(crate) fn get_code_from_nested(
    code: &str,
    nested_item_span: Span,
    parent_span: Option<Span>,
) -> String {
    let nested_item_code_start = if let Some(parent_span) = parent_span {
        nested_item_span.start().line - parent_span.start().line + 1
    } else {
        nested_item_span.start().line
    };

    let nested_item_code_end = if let Some(parent_span) = parent_span {
        nested_item_span.end().line - parent_span.start().line + 1
    } else {
        nested_item_span.end().line
    };

    let lines: Vec<&str> = code.lines().collect();

    if nested_item_code_start == nested_item_code_end {
        return lines[nested_item_code_start.saturating_sub(1)].to_string();
    }

    lines[(nested_item_code_start.saturating_sub(1))..nested_item_code_end].join("\n")
}

pub(crate) fn get_item_from_nested<I: ToTokens>(nested_item: I) -> SynItem {
    SynItem(
        parse_str(&nested_item.to_token_stream().to_string())
            .expect("Should always parse item from str"),
    )
}

pub(crate) fn get_type_ident(ty: &syn::Type) -> Option<syn::Ident> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return Some(seg.ident.clone());
        }
    }
    None
}
