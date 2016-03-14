use dom::bindings::reflector::Reflector;
use util::str::DOMString;
//use dom::bindings::codegen::UnionTypes::ElementOrProcessingInstruction;
use dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
//use dom::processinginstruction::ProcessingInstruction;

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

