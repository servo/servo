use dom::ProcessingInstruction;
use dom::bindings::reflector::Reflector;
use util::str::DOMString;

//#[dom_struct]
pub struct StyleSheet {
    type_: DOMString,
    reflector_: Reflector,
    /*href: DOMString,
    ownerNode: ProcessingInstruction,
    parentStyleSheet: StyleSheet,
    title: DOMString,
    media: MediaList,
    disabled: Cell<bool>,*/
}

impl StyleSheetMethods for StyleSheet{
    // http://dev.w3.org/csswg/cssom/#serialize-an-identifier
    pub fn type_(&self)-> &DOMString{
	&self.type_
    }
    /*pub fn href(&self)-> &DOMString{
	&self.href
    }
    pub fn title(&self)-> &DOMString{
	&self.title
    }
    pub fn parentStyleSheet(&self)-> StyleSheet{
	&self.parentStyleSheet
    }
    pub fn media(&self)-> MediaList{
	&self.media
    }
    pub fn disabled(&self)-> bool{
	&self.disabled
    }
    pub fn ownerNode(&self)-> ProcessingInstruction{
	&self.ownerNode
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

