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
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{
    FormDatum, FormDatumValue, FormSubmitterElement, HTMLFormElement,
};
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct FormData {
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

    pub(crate) fn new(
        form_datums: Option<Vec<FormDatum>>,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<FormData> {
        Self::new_with_proto(form_datums, global, None, can_gc)
    }

    fn new_with_proto(
        form_datums: Option<Vec<FormDatum>>,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<FormData> {
        reflect_dom_object_with_proto(
            Box::new(FormData::new_inherited(form_datums)),
            global,
            proto,
            can_gc,
        )
    }
}

impl FormDataMethods<crate::DomTypeHolder> for FormData {
    // https://xhr.spec.whatwg.org/#dom-formdata
    fn Constructor<'a>(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        form: Option<&'a HTMLFormElement>,
        submitter: Option<&'a HTMLElement>,
    ) -> Fallible<DomRoot<FormData>> {
        // Helper to validate the submitter
        fn validate_submitter<'b>(
            submitter: &'b HTMLElement,
            form: &'b HTMLFormElement,
        ) -> Result<FormSubmitterElement<'b>, Error> {
            let submit_button = submitter
                .downcast::<HTMLButtonElement>()
                .map(FormSubmitterElement::Button)
                .or_else(|| {
                    submitter
                        .downcast::<HTMLInputElement>()
                        .map(FormSubmitterElement::Input)
                })
                .ok_or(Error::Type(
                    "submitter is not a form submitter element".to_string(),
                ))?;

            // Step 1.1.1. If submitter is not a submit button, then throw a TypeError.
            if !submit_button.is_submit_button() {
                return Err(Error::Type("submitter is not a submit button".to_string()));
            }

            // Step 1.1.2. If submitter’s form owner is not form, then throw a "NotFoundError"
            // DOMException.
            if !matches!(submit_button.form_owner(), Some(owner) if *owner == *form) {
                return Err(Error::NotFound);
            }

            Ok(submit_button)
        }

        // Step 1. If form is given, then:
        if let Some(opt_form) = form {
            // Step 1.1. If submitter is non-null, then:
            let submitter_element = submitter
                .map(|s| validate_submitter(s, opt_form))
                .transpose()?;

            // Step 1.2. Let list be the result of constructing the entry list for form and submitter.
            return match opt_form.get_form_dataset(submitter_element, None, can_gc) {
                Some(form_datums) => Ok(FormData::new_with_proto(
                    Some(form_datums),
                    global,
                    proto,
                    can_gc,
                )),
                // Step 1.3. If list is null, then throw an "InvalidStateError" DOMException.
                None => Err(Error::InvalidState),
            };
        }

        Ok(FormData::new_with_proto(None, global, proto, can_gc))
    }

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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let datum = FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::File(DomRoot::from_ref(&*self.create_an_entry(
                blob,
                filename,
                CanGc::note(),
            ))),
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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let file = self.create_an_entry(blob, filename, CanGc::note());

        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name.0 != local_name);

        data.push((
            NoTrace(LocalName::from(name.0.clone())),
            FormDatum {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValue::File(file),
            },
        ));
    }
}

impl FormData {
    // https://xhr.spec.whatwg.org/#create-an-entry
    fn create_an_entry(
        &self,
        blob: &Blob,
        opt_filename: Option<USVString>,
        can_gc: CanGc,
    ) -> DomRoot<File> {
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
            can_gc,
        )
    }

    pub(crate) fn datums(&self) -> Vec<FormDatum> {
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
