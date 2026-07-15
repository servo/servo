/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::LocalName;
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use servo_constellation_traits::BlobImpl;

use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::UnionTypes::FileOrUSVString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::NoTrace;
use crate::dom::blob::Blob;
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlbuttonelement::HTMLButtonElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::{
    FormDatum, FormDatumUnrooted, FormDatumValueUnrooted, FormSubmitterElement, HTMLFormElement,
};
use crate::dom::html::input_element::HTMLInputElement;

#[dom_struct]
pub(crate) struct FormData {
    reflector_: Reflector,
    data: DomRefCell<Vec<(NoTrace<LocalName>, FormDatumUnrooted)>>,
}

impl FormData {
    fn new_inherited(form_datums: Option<Vec<FormDatum>>) -> FormData {
        FormData {
            reflector_: Reflector::new(),
            data: DomRefCell::new(
                form_datums
                    .into_iter()
                    .flatten()
                    .map(|datum| (NoTrace(LocalName::from(&datum.name)), datum.into()))
                    .collect::<Vec<(NoTrace<LocalName>, FormDatumUnrooted)>>(),
            ),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        form_datums: Option<Vec<FormDatum>>,
        global: &GlobalScope,
    ) -> DomRoot<FormData> {
        Self::new_with_proto(cx, form_datums, global, None)
    }

    fn new_with_proto(
        cx: &mut JSContext,
        form_datums: Option<Vec<FormDatum>>,
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<FormData> {
        reflect_dom_object_with_proto(
            cx,
            Box::new(FormData::new_inherited(form_datums)),
            global,
            proto,
        )
    }
}

impl FormDataMethods<crate::DomTypeHolder> for FormData {
    /// <https://xhr.spec.whatwg.org/#dom-formdata>
    fn Constructor<'a>(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
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
                    c"submitter is not a form submitter element".to_owned(),
                ))?;

            // Step 1.1.1. If submitter is not a submit button, then throw a TypeError.
            if !submit_button.is_submit_button() {
                return Err(Error::Type(c"submitter is not a submit button".to_owned()));
            }

            // Step 1.1.2. If submitter’s form owner is not form, then throw a "NotFoundError"
            // DOMException.
            if !matches!(submit_button.form_owner(), Some(owner) if *owner == *form) {
                return Err(Error::NotFound(None));
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
            return match opt_form.get_form_dataset(cx, submitter_element, None) {
                Some(form_datums) => Ok(FormData::new_with_proto(
                    cx,
                    Some(form_datums),
                    global,
                    proto,
                )),
                // Step 1.3. If list is null, then throw an "InvalidStateError" DOMException.
                None => Err(Error::InvalidState(None)),
            };
        }

        Ok(FormData::new_with_proto(cx, None, global, proto))
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-append>
    // As [`Append_`] needs &mut JSContext, we also need to have it for [`Append`]
    fn Append(&self, _cx: &mut JSContext, name: USVString, str_value: USVString) {
        self.data.borrow_mut().push((
            NoTrace(LocalName::from(name.0.clone())),
            FormDatumUnrooted {
                ty: DOMString::from("string"),
                name: DOMString::from(name.0),
                value: FormDatumValueUnrooted::String(DOMString::from(str_value.0)),
            },
        ));
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-append>
    fn Append_(
        &self,
        cx: &mut JSContext,
        name: USVString,
        blob: &Blob,
        filename: Option<USVString>,
    ) {
        let file = self.create_an_entry(cx, blob, filename);

        self.data.borrow_mut().push((
            NoTrace(LocalName::from(name.0.clone())),
            FormDatumUnrooted {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValueUnrooted::File(file.as_traced()),
            },
        ));
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-delete>
    fn Delete(&self, name: USVString) {
        self.data
            .borrow_mut()
            .retain(|(datum_name, _)| datum_name.0 != name.0);
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-get>
    fn Get(&self, name: USVString) -> Option<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .find(|(datum_name, _)| datum_name.0 == name.0)
            .map(|(_, datum)| match &datum.value {
                FormDatumValueUnrooted::String(s) => {
                    FileOrUSVString::USVString(USVString(s.to_string()))
                },
                FormDatumValueUnrooted::File(b) => FileOrUSVString::File(DomRoot::from_ref(b)),
            })
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-getall>
    fn GetAll(&self, name: USVString) -> Vec<FileOrUSVString> {
        self.data
            .borrow()
            .iter()
            .filter_map(|(datum_name, datum)| {
                if datum_name.0 != name.0 {
                    return None;
                }

                Some(match &datum.value {
                    FormDatumValueUnrooted::String(s) => {
                        FileOrUSVString::USVString(USVString(s.to_string()))
                    },
                    FormDatumValueUnrooted::File(b) => FileOrUSVString::File(DomRoot::from_ref(b)),
                })
            })
            .collect()
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-has>
    fn Has(&self, name: USVString) -> bool {
        self.data
            .borrow()
            .iter()
            .any(|(datum_name, _0)| datum_name.0 == name.0)
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-set>
    fn Set(&self, _cx: &mut JSContext, name: USVString, str_value: USVString) {
        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name.0 != local_name);

        data.push((
            NoTrace(local_name),
            FormDatumUnrooted {
                ty: DOMString::from("string"),
                name: DOMString::from(name.0),
                value: FormDatumValueUnrooted::String(DOMString::from(str_value.0)),
            },
        ));
    }

    /// <https://xhr.spec.whatwg.org/#dom-formdata-set>
    fn Set_(&self, cx: &mut JSContext, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let file = self.create_an_entry(cx, blob, filename);

        let mut data = self.data.borrow_mut();
        let local_name = LocalName::from(name.0.clone());

        data.retain(|(datum_name, _)| datum_name.0 != local_name);

        data.push((
            NoTrace(LocalName::from(name.0.clone())),
            FormDatumUnrooted {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValueUnrooted::File(file.as_traced()),
            },
        ));
    }
}

impl FormData {
    /// <https://xhr.spec.whatwg.org/#create-an-entry>
    fn create_an_entry(
        &self,
        cx: &mut JSContext,
        blob: &Blob,
        opt_filename: Option<USVString>,
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
        let last_modified = blob.downcast::<File>().map(|file| file.get_modified());

        File::new(
            cx,
            &self.global(),
            BlobImpl::new_from_bytes(bytes, blob.type_string()),
            name,
            last_modified,
        )
    }

    pub(crate) fn datums(&self) -> Vec<FormDatum> {
        self.data
            .borrow()
            .iter()
            .map(|(_, datum)| datum.root())
            .collect()
    }
}

impl Iterable for FormData {
    type Key = USVString;
    type Value = FileOrUSVString;

    fn get_iterable_length(&self, _cx: &mut JSContext) -> u32 {
        self.data.borrow().len() as u32
    }

    fn get_value_at_index(&self, _cx: &mut JSContext, index: u32) -> FileOrUSVString {
        let data = self.data.borrow();
        let datum = &data.get(index as usize).unwrap().1;
        match &datum.value {
            FormDatumValueUnrooted::String(s) => {
                FileOrUSVString::USVString(USVString(s.to_string()))
            },
            FormDatumValueUnrooted::File(b) => FileOrUSVString::File(DomRoot::from_ref(b)),
        }
    }

    fn get_key_at_index(&self, _cx: &mut JSContext, index: u32) -> USVString {
        let data = self.data.borrow();
        let key = &data.get(index as usize).unwrap().0;
        USVString(key.to_string())
    }
}
