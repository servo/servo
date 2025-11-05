use rustc_hash::FxHashMap;
use base::id::NamespaceIndex;
use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::MediaSourceHandleBinding::MediaSourceHandle_Binding::MediaSourceHandleMethods;
use script_bindings::error::Fallible;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct MediaSourceHandle {
    reflector_: Reflector,
}

impl Transferable for MediaSourceHandle {
    type Index = ();
    type Data = ();

    fn transfer(&self) -> Fallible<(NamespaceIndex<Self::Index>, Self::Data)> {
        todo!()
    }

    fn transfer_receive(
        _owner: &GlobalScope,
        _id: NamespaceIndex<Self::Index>,
        _serialized: Self::Data,
    ) -> Result<DomRoot<Self>, ()> {
        todo!()
    }

    fn serialized_storage<'a>(
        _data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<NamespaceIndex<Self::Index>, Self::Data>> {
        todo!()
    }
}

impl MediaSourceHandleMethods<crate::DomTypeHolder> for MediaSourceHandle {}
