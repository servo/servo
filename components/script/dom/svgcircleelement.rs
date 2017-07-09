use dom::attr::Attr;
use dom::bindings::codegen::Bindings::SVGCircleElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::node::Node;
use dom::svgelement::SVGElement;
use dom::svggeometryelement::SVGGeometryElement;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::ElementState;

#[dom_struct]
pub struct SVGCircleElement {
    geometryelement: SVGGeometryElement,
}

impl SVGCircleElement {
    pub fn new_inherited(tag_name: LocalName,
                         prefix: Option<Prefix>,
                         document: &Document)
                         -> Self {
        SVGCircleElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState,
                                    tag_name: LocalName,
                                    prefix: Option<Prefix>,
                                    document: &Document)
                                    -> Self {
        SVGCircleElement {
            geometryelement: SVGGeometryElement::new_inherited_with_state(state,
                                                                          tag_name,
                                                                          prefix,
                                                                          document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<Prefix>, document: &Document)
               -> Root<SVGCircleElement> {
        Node::reflect_node(box SVGCircleElement::new_inherited(local_name, prefix, document),
                           document,
                           SVGCircleElementBinding::Wrap)
    }
}

impl VirtualMethods for SVGCircleElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<SVGGeometryElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        let name = attr.local_name();

        match name {
            &local_name!("cx") => {
                self.upcast::<SVGElement>().mutate_svg_preshint(attr);
            }
            &local_name!("cy") => {
                self.upcast::<SVGElement>().mutate_svg_preshint(attr);
            }
            &local_name!("r") => {
                self.upcast::<SVGElement>().mutate_svg_preshint(attr);
            }
            _ => {}
        }
    }
}
