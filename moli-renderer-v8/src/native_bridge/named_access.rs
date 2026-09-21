use crate::document_runtime::DomHandle;
use moli_dom::native::DomHost;

use super::{
    JsContextHost, collections,
    identity::{CollectionKind, LiveCollectionQueryKind},
};

const WINDOW_NAME_ELEMENTS: &[&str] = &["img", "form", "embed", "object"];
const DOCUMENT_NAME_ELEMENTS: &[&str] = &["embed", "form", "iframe", "img", "object"];
const DOCUMENT_ALL_NAME_ELEMENTS: &[&str] = &[
    "a", "button", "embed", "form", "frame", "frameset", "iframe", "img", "input", "map", "meta",
    "object", "select", "textarea",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyNamedAccessKind {
    Window,
    Document,
    DocumentAll,
}

pub(crate) fn html_element_name_attribute_is_exposed(
    local_name: &str,
    kind: LegacyNamedAccessKind,
) -> bool {
    name_elements(kind).contains(&local_name)
}

pub(crate) fn dom_element_name_attribute_is_exposed(
    dom: &DomHost,
    handle: DomHandle,
    kind: LegacyNamedAccessKind,
) -> bool {
    name_elements(kind)
        .iter()
        .any(|local_name| dom.is_html_element_named(handle, local_name))
}

pub(crate) fn window_named_item_handles(
    dom: &DomHost,
    root: DomHandle,
    name: &str,
) -> Vec<DomHandle> {
    dom.element_handles_by_id_or_name_matching_in_subtree(root, name, |handle| {
        dom_element_name_attribute_is_exposed(dom, handle, LegacyNamedAccessKind::Window)
    })
}

pub(crate) fn document_named_item_handles(
    dom: &DomHost,
    root: DomHandle,
    name: &str,
) -> Vec<DomHandle> {
    if name.is_empty() {
        return Vec::new();
    }

    dom.element_handles_by_id_or_name_matching_in_subtree(root, name, |handle| {
        dom_element_name_attribute_is_exposed(dom, handle, LegacyNamedAccessKind::Document)
    })
    .into_iter()
    .filter(|handle| {
        let Some(element) = dom.node(*handle).and_then(crate::dom::native::Node::as_element)
        else {
            return false;
        };
        let local_name = element.local_name();
        let name_matches = element.name_attribute() == Some(name)
            && DOCUMENT_NAME_ELEMENTS.contains(&local_name);
        let id_matches = element.id() == Some(name)
            && (local_name == "object"
                || (local_name == "img"
                    && element
                        .name_attribute()
                        .is_some_and(|value| !value.is_empty())));
        name_matches || id_matches
    })
    .collect()
}

pub(crate) fn document_all_named_item_handles(dom: &DomHost, name: &str) -> Vec<DomHandle> {
    dom.element_handles_by_id_or_name_matching(name, |handle| {
        dom_element_name_attribute_is_exposed(dom, handle, LegacyNamedAccessKind::DocumentAll)
    })
}

pub(crate) fn build_window_named_items_collection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    runtime_ptr: *mut JsContextHost,
    root: DomHandle,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    Some(collections::build_live_collection_for_node(
        scope,
        runtime_ptr,
        root,
        CollectionKind::HtmlCollection,
        LiveCollectionQueryKind::WindowNamedItems,
        Some(name.to_owned()),
        false,
    ))
}

pub(crate) fn build_document_named_items_collection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    runtime_ptr: *mut JsContextHost,
    root: DomHandle,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    Some(collections::build_live_collection_for_node(
        scope,
        runtime_ptr,
        root,
        CollectionKind::HtmlCollection,
        LiveCollectionQueryKind::DocumentNamedItems,
        Some(name.to_owned()),
        false,
    ))
}

fn name_elements(kind: LegacyNamedAccessKind) -> &'static [&'static str] {
    match kind {
        LegacyNamedAccessKind::Window => WINDOW_NAME_ELEMENTS,
        LegacyNamedAccessKind::Document => DOCUMENT_NAME_ELEMENTS,
        LegacyNamedAccessKind::DocumentAll => DOCUMENT_ALL_NAME_ELEMENTS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_name_attribute_candidates_are_scoped_by_consumer() {
        for local_name in ["img", "form", "embed", "object"] {
            assert!(html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::Window
            ));
        }
        for local_name in ["a", "button", "iframe", "input", "meta", "applet", "div"] {
            assert!(!html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::Window
            ));
        }
        for local_name in ["embed", "form", "iframe", "img", "object"] {
            assert!(html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::Document
            ));
        }
        for local_name in ["a", "button", "input", "meta", "select", "textarea", "div"] {
            assert!(!html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::Document
            ));
        }
        for local_name in [
            "a", "button", "embed", "form", "frame", "frameset", "iframe", "img", "input", "map",
            "meta", "object", "select", "textarea",
        ] {
            assert!(html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::DocumentAll
            ));
        }
        for local_name in ["applet", "div", "svg"] {
            assert!(!html_element_name_attribute_is_exposed(
                local_name,
                LegacyNamedAccessKind::DocumentAll
            ));
        }
    }
}
