use content::content_task::global_content;
use dom::bindings::utils::{DOMString, ErrorResult, WrapperCache, CacheableWrapper};
use dom::bindings::codegen::DOMParserBinding;
use dom::document::Document;
use dom::element::{Element, HTMLHtmlElement, HTMLHtmlElementTypeId};
use dom::node::Node;
use dom::window::Window;

pub struct DOMParser {
    owner: @mut Window, //XXXjdm Document instead?
    wrapper: WrapperCache
}

pub impl DOMParser {
    fn new(owner: @mut Window) -> @mut DOMParser {
        let parser = @mut DOMParser {
            owner: owner,
            wrapper: WrapperCache::new()
        };
        let cx = global_content().compartment.get().cx.ptr;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        parser.wrap_object_shared(cx, scope);
        parser
    }

    fn Constructor(owner: @mut Window, _rv: &mut ErrorResult) -> @mut DOMParser {
        DOMParser::new(owner)
    }

    fn ParseFromString(&self, _s: DOMString, _type_: DOMParserBinding::SupportedType, _rv: &mut ErrorResult) -> @mut Document {
        let root = ~HTMLHtmlElement { parent: Element::new(HTMLHtmlElementTypeId, ~"html") };
        let root = unsafe { Node::as_abstract_node(root) };
        Document(root, None)
    }
}