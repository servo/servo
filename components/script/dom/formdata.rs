/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use crate::dom::bindings::codegen::Bindings::FormDataBinding::FormDataWrap;
use crate::dom::bindings::codegen::UnionTypes::FileOrUSVString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::iterable::Iterable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::blob::{Blob, BlobImpl};
use crate::dom::file::File;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlformelement::{FormDatum, FormDatumValue, HTMLFormElement};
use dom_struct::dom_struct;
use html5ever::LocalName;
use std::collections::btree_map::Entry::{Occupied, Vacant};
use std::collections::BTreeMap;
use std::iter;

#[dom_struct]
pub struct FormData {
    reflector_: Reflector,
    data: DomRefCell<BTreeMap<LocalName, Vec<FormDatum>>>,
}

impl FormData {
    fn new_inherited(form_datums: Option<Vec<FormDatum>>) -> FormData {
        let mut hashmap: BTreeMap<LocalName, Vec<FormDatum>> = BTreeMap::new();

        if let Some(form_datums) = form_datums {
            for datum in form_datums {
                match hashmap.entry(LocalName::from(datum.name.as_ref())) {
                    Occupied(entry) => entry.into_mut().push(datum),
                    Vacant(entry) => {
                        entry.insert(vec![datum]);
                    },
                }
            }
        }

        FormData {
            reflector_: Reflector::new(),
            data: DomRefCell::new(hashmap),
        }
    }

    pub fn new(form_datums: Option<Vec<FormDatum>>, global: &GlobalScope) -> DomRoot<FormData> {
        reflect_dom_object(
            Box::new(FormData::new_inherited(form_datums)),
            global,
            FormDataWrap,
        )
    }

    // https://xhr.spec.whatwg.org/#dom-formdata
    pub fn Constructor(
        global: &GlobalScope,
        form: Option<&HTMLFormElement>,
    ) -> Fallible<DomRoot<FormData>> {
        if let Some(opt_form) = form {
            return match opt_form.get_form_dataset(None) {
                Some(form_datums) => Ok(FormData::new(Some(form_datums), global)),
                None => Err(Error::InvalidState),
            };
        }

        Ok(FormData::new(None, global))
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

        let mut data = self.data.borrow_mut();
        match data.entry(LocalName::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(datum),
            Vacant(entry) => {
                entry.insert(vec![datum]);
            },
        }
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-append
    fn Append_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        let datum = FormDatum {
            ty: DOMString::from("file"),
            name: DOMString::from(name.0.clone()),
            value: FormDatumValue::File(DomRoot::from_ref(&*self.create_an_entry(blob, filename))),
        };

        let mut data = self.data.borrow_mut();

        match data.entry(LocalName::from(name.0)) {
            Occupied(entry) => entry.into_mut().push(datum),
            Vacant(entry) => {
                entry.insert(vec![datum]);
            },
        }
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-delete
    fn Delete(&self, name: USVString) {
        self.data.borrow_mut().remove(&LocalName::from(name.0));
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-get
    fn Get(&self, name: USVString) -> Option<FileOrUSVString> {
        self.data
            .borrow()
            .get(&LocalName::from(name.0))
            .map(|entry| match entry[0].value {
                FormDatumValue::String(ref s) => {
                    FileOrUSVString::USVString(USVString(s.to_string()))
                },
                FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(&*b)),
            })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-getall
    fn GetAll(&self, name: USVString) -> Vec<FileOrUSVString> {
        self.data
            .borrow()
            .get(&LocalName::from(name.0))
            .map_or(vec![], |data| {
                data.iter()
                    .map(|item| match item.value {
                        FormDatumValue::String(ref s) => {
                            FileOrUSVString::USVString(USVString(s.to_string()))
                        },
                        FormDatumValue::File(ref b) => {
                            FileOrUSVString::File(DomRoot::from_ref(&*b))
                        },
                    })
                    .collect()
            })
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-has
    fn Has(&self, name: USVString) -> bool {
        self.data.borrow().contains_key(&LocalName::from(name.0))
    }

    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set(&self, name: USVString, str_value: USVString) {
        self.data.borrow_mut().insert(
            LocalName::from(name.0.clone()),
            vec![FormDatum {
                ty: DOMString::from("string"),
                name: DOMString::from(name.0),
                value: FormDatumValue::String(DOMString::from(str_value.0)),
            }],
        );
    }

    #[allow(unrooted_must_root)]
    // https://xhr.spec.whatwg.org/#dom-formdata-set
    fn Set_(&self, name: USVString, blob: &Blob, filename: Option<USVString>) {
        self.data.borrow_mut().insert(
            LocalName::from(name.0.clone()),
            vec![FormDatum {
                ty: DOMString::from("file"),
                name: DOMString::from(name.0),
                value: FormDatumValue::File(DomRoot::from_ref(
                    &*self.create_an_entry(blob, filename),
                )),
            }],
        );
    }
}

impl FormData {
    // https://xhr.spec.whatwg.org/#create-an-entry
    // Steps 3-4.
    fn create_an_entry(&self, blob: &Blob, opt_filename: Option<USVString>) -> DomRoot<File> {
        let name = match opt_filename {
            Some(filename) => DOMString::from(filename.0),
            None if blob.downcast::<File>().is_none() => DOMString::from("blob"),
            None => DOMString::from(""),
        };

        let bytes = blob.get_bytes().unwrap_or(vec![]);

        File::new(
            &self.global(),
            BlobImpl::new_from_bytes(bytes),
            name,
            None,
            &blob.type_string(),
        )
    }

    pub fn datums(&self) -> Vec<FormDatum> {
        self.data
            .borrow()
            .values()
            .flat_map(|value| value.iter())
            .map(|value| value.clone())
            .collect()
    }
}

impl Iterable for FormData {
    type Key = USVString;
    type Value = FileOrUSVString;

    fn get_iterable_length(&self) -> u32 {
        self.data
            .borrow()
            .values()
            .map(|value| value.len())
            .sum::<usize>() as u32
    }

    fn get_value_at_index(&self, n: u32) -> FileOrUSVString {
        let data = self.data.borrow();
        let value = &data
            .values()
            .flat_map(|value| value.iter())
            .nth(n as usize)
            .unwrap()
            .value;
        match *value {
            FormDatumValue::String(ref s) => FileOrUSVString::USVString(USVString(s.to_string())),
            FormDatumValue::File(ref b) => FileOrUSVString::File(DomRoot::from_ref(&*b)),
        }
    }

    fn get_key_at_index(&self, n: u32) -> USVString {
        let data = self.data.borrow();
        let value = &data
            .iter()
            .flat_map(|(key, value)| iter::repeat(key).take(value.len()))
            .nth(n as usize)
            .unwrap();
        USVString(value.to_string())
    }
}
