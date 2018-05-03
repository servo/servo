use typeholder::TypeHolderTrait;
use dom::bindings::reflector::DomObject;
use dom::bindings::reflector::MutDomObject;
use malloc_size_of::MallocSizeOf;
use dom::bindings::trace::JSTraceable;
use dom::bindings::conversions::IDLInterface;
use dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserBinding::DOMParserMethods;
use dom::window::Window;
use dom::bindings::error::Fallible;
use dom::bindings::root::DomRoot;

pub trait DOMParserTrait<TH: TypeHolderTrait>: DomObject + MutDomObject + IDLInterface + MallocSizeOf + JSTraceable + DOMParserMethods<TH> {
    fn Constructor(window: &Window<TH>) -> Fallible<DomRoot<TH::DOMParser>>;
}