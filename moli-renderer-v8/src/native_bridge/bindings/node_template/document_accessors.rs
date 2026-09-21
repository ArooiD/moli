use crate::context_bootstrap::bridge_descriptor::BridgeDescriptor;
use crate::native_bridge::document::{
    document_named_property_getter, document_named_property_query,
};

pub(super) fn install_document_accessors<'s, 'i>(
    _scope: &mut v8::PinScope<'s, 'i, ()>,
    template: v8::Local<'s, v8::ObjectTemplate>,
    descriptor: &BridgeDescriptor,
) {
    if !matches!(descriptor.interface.name(), "Document" | "HTMLDocument") {
        return;
    }

    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(document_named_property_getter)
            .query(document_named_property_query)
            .flags(
                v8::PropertyHandlerFlags::NON_MASKING
                    | v8::PropertyHandlerFlags::ONLY_INTERCEPT_STRINGS,
            ),
    );
}
