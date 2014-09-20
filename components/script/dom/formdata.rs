/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::Bindings::FormDataBinding::FormDataMethods;
use dom::bindings::codegen::InheritTypes::FileCast;
use dom::bindings::codegen::UnionTypes::FileOrString::{FileOrString, eFile, eString};
use dom::bindings::error::{Fallible};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::blob::Blob;
use dom::file::File;
use dom::htmlformelement::HTMLFormElement;
use servo_util::str::DOMString;
use std::cell::RefCell;
use std::collections::hashmap::HashMap;

#[deriving(Encodable, Clone)]
#[must_root]
pub enum FormDatum {
    StringData(DOMString),
    FileData(JS<File>)
}

#[deriving(Encodable)]
#[must_root]
pub struct FormData {
    data: Traceable<RefCell<HashMap<DOMString, Vec<FormDatum>>>>,
    reflector_: Reflector,
    global: GlobalField,
    form: Option<JS<HTMLFormElement>>
}

impl FormData {
    pub fn new_inherited(form: Option<JSRef<HTMLFormElement>>, global: &GlobalRef) -> FormData {
        FormData {
            data: Traceable::new(RefCell::new(HashMap::new())),
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
            form: form.map(|f| JS::from_rooted(f)),
        }
    }

    pub fn new(form: Option<JSRef<HTMLFormElement>>, global: &GlobalRef) -> Temporary<FormData> {
        reflect_dom_object(box FormData::new_inherited(form, global),
                           global, FormDataBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef, form: Option<JSRef<HTMLFormElement>>) -> Fallible<Temporary<FormData>> {
        Ok(FormData::new(form, global))
    }
}

impl<'a> FormDataMethods for JSRef<'a, FormData> {
    #[allow(unrooted_must_root)]
    fn Append(self, name: DOMString, value: JSRef<Blob>, filename: Option<DOMString>) {
        let file = FileData(JS::from_rooted(self.get_file_from_blob(value, filename)));
        self.data.deref().borrow_mut().insert_or_update_with(name.clone(), vec!(file.clone()),
                                        |_k, v| {v.push(file.clone());});
    }

    fn Append_(self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert_or_update_with(name, vec!(StringData(value.clone())),
                                        |_k, v| {v.push(StringData(value.clone()));});
    }

    fn Delete(self, name: DOMString) {
        self.data.deref().borrow_mut().remove(&name);
    }

    fn Get(self, name: DOMString) -> Option<FileOrString> {
        if self.data.deref().borrow().contains_key_equiv(&name) {
            match self.data.deref().borrow().get(&name)[0].clone() {
                StringData(ref s) => Some(eString(s.clone())),
                FileData(ref f) => {
                    Some(eFile(f.clone()))
                }
            }
        } else {
            None
        }
    }

    fn Has(self, name: DOMString) -> bool {
        self.data.deref().borrow().contains_key_equiv(&name)
    }
    #[allow(unrooted_must_root)]
    fn Set(self, name: DOMString, value: JSRef<Blob>, filename: Option<DOMString>) {
        let file = FileData(JS::from_rooted(self.get_file_from_blob(value, filename)));
        self.data.deref().borrow_mut().insert(name, vec!(file));
    }

    fn Set_(self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert(name, vec!(StringData(value)));
    }
}

impl Reflectable for FormData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

trait PrivateFormDataHelpers{
  fn get_file_from_blob(&self, value: JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File>;
}

impl PrivateFormDataHelpers for FormData {
    fn get_file_from_blob(&self, value: JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File> {
        let global = self.global.root();
        let f: Option<JSRef<File>> = FileCast::to_ref(value);
        let name = filename.unwrap_or(f.map(|inner| inner.name.clone()).unwrap_or("blob".to_string()));
        File::new(&global.root_ref(), value, name)
    }
}
