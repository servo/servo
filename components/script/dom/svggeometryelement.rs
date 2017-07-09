use dom::bindings::inheritance::Castable;
use dom::document::Document;
use dom::svggraphicselement::SVGGraphicsElement;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::ElementState;

#[dom_struct]
pub struct SVGGeometryElement {
    graphicselement: SVGGraphicsElement
}

impl SVGGeometryElement {
    pub fn new_inherited(tag_name: LocalName, prefix: Option<Prefix>,
                         document: &Document) -> Self {
        SVGGeometryElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, tag_name: LocalName,
                                    prefix: Option<Prefix>, document: &Document)
                                    -> Self {
        SVGGeometryElement {
            graphicselement:
                SVGGraphicsElement::new_inherited_with_state(state, tag_name, prefix, document),
        }
    }
}

impl VirtualMethods for SVGGeometryElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<SVGGraphicsElement>() as &VirtualMethods)
    }
}
