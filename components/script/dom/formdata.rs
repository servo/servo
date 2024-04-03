/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::LocalName;
use js::rust::HandleObject;
use script_traits::serializable::BlobImpl;

use super::bindings::trace::NoTrace;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::UnionTypes::FileOrUSVString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::{FormDatum, FormDatumValue, HTMLFormElement};

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DomRefCell<Vec<(NoTrace<LocalName>, FormDatum)>>,
}

impl FormData {
    fn new_inherited(form_datums: Option<Vec<FormDatum>>) -> FormData {
        let data = match form_datums {
            Some(data) => data
                .iter()
                .map(|datum| (NoTrace(LocalName::from(datum.name.as_ref())), datum.clone()))
                .collect::<Vec<(NoTrace<LocalName>, FormDatum)>>(),
            None => Vec::new(),
        };

        FormData {
            reflector_: Reflector::new(),
            data: DomRefCell::new(data),
        }
    }

    pub fn new(form_datums: Option<Vec<FormDatum>>, global: &GlobalScope) -> DomRoot<FormData> {
        Self::new_with_proto(form_datums, global, None)
    }

    fn new_with_proto(
        form_datums: Option<Vec<FormDatum>>,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<FormData> {
        reflect_dom_object_with_proto(
            Box::new(FormData::new_inherited(form_datums)),
            global,
            proto,
        )
    }

    // https://xhr.spec.whatwg.org/#dom-formdata
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        form: Option<&HTMLFormElement>,
    ) -> Fallible<DomRoot<FormData>> {
        if let Some(opt_form) = form {
            return match opt_form.get_form_dataset(None, None) {
                Some(form_datums) => Ok(FormData::new_with_proto(Some(form_datums), global, proto)),
                None => Err(Error::InvalidState),
            };
        }

        Ok(FormData::new_with_proto(None, global, proto))
    }
}

impl FormDataMethods for FormData {
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append(&self, name: USVString, str_value: USVString) {
        let datum = FormDatum {
            ty: DOMString::from("string"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::String(DOMString::from(str_value.0)),
        };

        self.data
            .borrow_mut()
            .push((NoTrace(LocalName::from(name.0)), datum));
    }

    #[allow(crown::unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let datum = FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::File(DomRoot::from_ref(&*self.create_an_entry(blob, filename))),
        };

        self.data
            .borrow_mut()
            .push((NoTrace(LocalName::from(name.0)), datum));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: USVString) {
        self.data
            .borrow_mut()
            .retain(|(datum_name, _)| datum_name.0 != name.0);
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: USVString) -> Option<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .find(|(datum_name, _)| datum_name.0 == name.0)
            .map(|(_, datum)| match &datum.value {
                FormDatumValue::String(ref s) => {
                    FileOrUSVString::USVString(USVString(s.to_string()))
                },
                FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(b)),
            })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-getall
    fn GetAll(&self, name: USVString) -> Vec<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .filter_map(|(datum_name, datum)| {
                if datum_name.0 != name.0 {
                    return None;
                }

                Some(match &datum.value {
                    FormDatumValue::String(ref s) => {
                        FileOrUSVString::USVString(USVString(s.to_string()))
                    },
                    FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(b)),
                })
            })
            .collect()
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: USVString) -> bool {
        self.data
            .borrow()
            .iter()
            .any(|(datum_name, _0)| datum_name.0 == name.0)
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: USVString, str_value: USVString) {
        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name.0 != local_name);

        data.push((
            NoTrace(local_name),
            FormDatum {
                ty: DOMString::from("string"),
                name: DOMString::from(name.0),
                value: FormDatumValue::String(DOMString::from(str_value.0)),
            },
        ));
    }

    #[allow(crown::unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name.0 != local_name);

        data.push((
            NoTrace(LocalName::from(name.0.clone())),
            FormDatum {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValue::File(DomRoot::from_ref(
                    &*self.create_an_entry(blob, filename),
                )),
            },
        ));
    }
}

impl FormData {
    // https://xhr.spec.whatwg.org/#create-an-entry
    fn create_an_entry(&self, blob: &Blob, opt_filename: Option<USVString>) -> DomRoot<File> {
        // Steps 3-4
        let name = match opt_filename {
            Some(filename) => DOMString::from(filename.0),
            None => match blob.downcast::<File>() {
                None => DOMString::from("blob"),
                // If it is already a file and no filename was given,
                // then neither step 3 nor step 4 happens, so instead of
                // creating a new File object we use the existing one.
                Some(file) => {
                    return DomRoot::from_ref(file);
                },
            },
        };

        let bytes = blob.get_bytes().unwrap_or_default();

        File::new(
            &self.global(),
            BlobImpl::new_from_bytes(bytes, blob.type_string()),
            name,
            None,
        )
    }

    pub fn datums(&self) -> Vec<FormDatum> {
        self.data
            .borrow()
            .iter()
            .map(|(_, datum)| datum.clone())
            .collect()
    }
}

impl Iterable for FormData {
    type Key = USVString;
    type Value = FileOrUSVString;

    fn get_iterable_length(&self) -> u32 {
        self.data.borrow().len() as u32
    }

    fn get_value_at_index(&self, n: u32) -> FileOrUSVString {
        let data = self.data.borrow();
        let datum = &data.get(n as usize).unwrap().1;
        match &datum.value {
            FormDatumValue::String(ref s) => FileOrUSVString::USVString(USVString(s.to_string())),
            FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(b)),
        }
    }

    fn get_key_at_index(&self, n: u32) -> USVString {
        let data = self.data.borrow();
        let key = &data.get(n as usize).unwrap().0;
        USVString(key.to_string())
    }
}
