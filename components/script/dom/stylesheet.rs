//use dom::bindings::reflector::Reflector;
use util::str::DOMString;
//use dom::bindings::codegen::UnionTypes::ElementOrProcessingInstruction;
use dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
//use dom::processinginstruction::ProcessingInstruction;
use dom::bindings::codegen::Bindings::StyleSheetBinding;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
//use dom::document::Document;
//use dom::stylesheet::StyleSheet;
use dom::window::Window;


#[dom_struct]
pub struct StyleSheet {
    reflector_: Reflector,
    type_: DOMString,
    href: Option<DOMString>,
    title: Option<DOMString>,
    //ownerNode: Option<ProcessingInstruction>,
    /*parentStyleSheet: StyleSheet,
    parentStyleSheet: StyleSheet,
    
    media: MediaList,
    disabled: Cell<bool>,*/
}

impl StyleSheet {
    #[allow(unrooted_must_root)]
    fn new_inherited(type_: DOMString, href: Option<DOMString>, title: Option<DOMString>) -> StyleSheet {
        StyleSheet {
            reflector_: Reflector::new(),
            type_: type_,
            href: href,
            title: title
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, type_: DOMString, href: Option<DOMString>, title: Option<DOMString>) -> Root<StyleSheet> {
        reflect_dom_object(box StyleSheet::new_inherited(type_,href,title),
                           GlobalRef::Window(window), StyleSheetBinding::Wrap)
    }
}


impl StyleSheetMethods for StyleSheet {
    // http://dev.w3.org/csswg/cssom/#serialize-an-identifier
    fn Type_(&self)-> DOMString{
	self.type_.clone()
    }
    fn GetHref(&self)-> Option<DOMString>{
	self.href.clone()
    }
    fn GetTitle(&self)-> Option<DOMString>{
	self.title.clone()
    }/*
    fn GetOwnerNode(&self) -> Option<ProcessingInstruction>{
	self.ownerNode
    }
    fn Disabled(&self)-> bool{
	self.disabled.clone()
    }
    fn parentStyleSheet(&self)-> &StyleSheet{
	&self.parentStyleSheet
    }
    pub fn media(&self)-> MediaList{
	&self.media
    }
    pub fn set_disable(&self,val: bool) -> StyleSheet{
        if val{
          &self.disabled = true
        } else {
          &self.disabled = false
	}
        &self
    }*/
	
}

