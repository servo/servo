/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FormDataBinding;
use dom::bindings::codegen::InheritTypes::FileCast;
use dom::bindings::codegen::UnionTypes::FileOrString::{FileOrString, eFile, eString};
use dom::bindings::error::{Fallible};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::blob::Blob;
use dom::file::File;
use dom::htmlformelement::HTMLFormElement;
use dom::window::Window;
use servo_util::str::DOMString;
use std::cell::RefCell;
use std::collections::hashmap::HashMap;

#[deriving(Encodable, Clone)]
pub enum FormDatum {
    StringData(DOMString),
    FileData(JS<File>)
}

#[deriving(Encodable)]
pub struct FormData {
    pub data: Traceable<RefCell<HashMap<DOMString, Vec<FormDatum>>>>,
    pub reflector_: Reflector,
    pub window: JS<Window>,
    pub form: Option<JS<HTMLFormElement>>
}

impl FormData {
    pub fn new_inherited(form: Option<JSRef<HTMLFormElement>>, window: &JSRef<Window>) -> FormData {
        FormData {
            data: Traceable::new(RefCell::new(HashMap::new())),
            reflector_: Reflector::new(),
            window: JS::from_rooted(window),
            form: form.map(|f| JS::from_rooted(&f)),
        }
    }

    pub fn new(form: Option<JSRef<HTMLFormElement>>, window: &JSRef<Window>) -> Temporary<FormData> {
        reflect_dom_object(box FormData::new_inherited(form, window), window, FormDataBinding::Wrap)
    }

    pub fn Constructor(window: &JSRef<Window>, form: Option<JSRef<HTMLFormElement>>) -> Fallible<Temporary<FormData>> {
        Ok(FormData::new(form, window))
    }
}

pub trait FormDataMethods {
    fn Append(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>);
    fn Append_(&self, name: DOMString, value: DOMString);
    fn Delete(&self, name: DOMString);
    fn Get(&self, name: DOMString) -> Option<FileOrString>;
    fn Has(&self, name: DOMString) -> bool;
    fn Set(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>);
    fn Set_(&self, name: DOMString, value: DOMString);
}

impl<'a> FormDataMethods for JSRef<'a, FormData> {
    fn Append(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>) {
        let file = FileData(JS::from_rooted(&self.get_file_from_blob(value, filename)));
        self.data.deref().borrow_mut().insert_or_update_with(name.clone(), vec!(file.clone()),
                                        |_k, v| {v.push(file.clone());});
    }

    fn Append_(&self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert_or_update_with(name, vec!(StringData(value.clone())),
                                        |_k, v| {v.push(StringData(value.clone()));});
    }

    fn Delete(&self, name: DOMString) {
        self.data.deref().borrow_mut().remove(&name);
    }

    fn Get(&self, name: DOMString) -> Option<FileOrString> {
        if self.data.deref().borrow().contains_key_equiv(&name) {
            match self.data.deref().borrow().get(&name).get(0).clone() {
                StringData(ref s) => Some(eString(s.clone())),
                FileData(ref f) => {
                    Some(eFile(f.clone()))
                }
            }
        } else {
            None
        }
    }

    fn Has(&self, name: DOMString) -> bool {
        self.data.deref().borrow().contains_key_equiv(&name)
    }

    fn Set(&self, name: DOMString, value: &JSRef<Blob>, filename: Option<DOMString>) {
        let file = FileData(JS::from_rooted(&self.get_file_from_blob(value, filename)));
        self.data.deref().borrow_mut().insert(name, vec!(file));
    }

    fn Set_(&self, name: DOMString, value: DOMString) {
        self.data.deref().borrow_mut().insert(name, vec!(StringData(value)));
    }
}

impl Reflectable for FormData {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

trait PrivateFormDataHelpers{
  fn get_file_from_blob(&self, value: &JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File>;
}

impl PrivateFormDataHelpers for FormData {
    fn get_file_from_blob(&self, value: &JSRef<Blob>, filename: Option<DOMString>) -> Temporary<File> {
        let global = self.window.root();
        let f: Option<&JSRef<File>> = FileCast::to_ref(value);
        let name = filename.unwrap_or(f.map(|inner| inner.name.clone()).unwrap_or("blob".to_string()));
        File::new(&*global, value, name)
    }
}
