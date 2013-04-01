use dom::bindings::utils::{DOMString, ErrorResult, WrapperCache};
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
    fn new(owner: @mut Window) -> DOMParser {
        DOMParser {
            owner: owner,
            wrapper: WrapperCache::new()
        }
    }

    fn Constructor(owner: @mut Window, _rv: &mut ErrorResult) -> @mut DOMParser {
        @mut DOMParser::new(owner)
    }

    fn ParseFromString(&self, _s: DOMString, _type_: DOMParserBinding::SupportedType, _rv: &mut ErrorResult) -> @mut Document {
        let root = ~HTMLHtmlElement { parent: Element::new(HTMLHtmlElementTypeId, ~"html") };
        let root = unsafe { Node::as_abstract_node(root) };
        @mut Document(root)
    }
}