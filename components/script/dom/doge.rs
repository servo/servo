/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DogeBinding::{DogeMethods, DogeInit};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::globalscope::GlobalScope;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::str::ByteString;
use servo_rand;
use servo_rand::Rng;
use dom_struct::dom_struct;

#[dom_struct]
pub struct Doge {
    reflector_: Reflector,
    such_list: DomRefCell<Vec<ByteString>>,
}

impl Doge {
    pub fn new_inherited() -> Doge {
        Doge {
            reflector_: Reflector::new(),
            such_list: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<Doge> {
        reflect_dom_object(Box::new(Doge::new_inherited()), global)
    }

    
    pub fn Constructor(global: &GlobalScope, init: Option<DogeInit>) -> Fallible<DomRoot<Doge>> {
        let doge = Doge::new(global);
        if let Some(i) = init {
            for word in i {
                doge.Append(word);
            }
        }
        
        Ok(doge)
    }
}

impl DogeMethods for Doge {
    fn Append(&self, word: ByteString) -> () {
        *&self.such_list.borrow_mut().push(word);
    }

    fn Random(&self) -> Fallible<ByteString> {
        let list = self.such_list.borrow();
        if list.len() == 0 {
            return Err(Error::Type("Such list is empty".to_string()));
        } else {
            let random_index = servo_rand::thread_rng().gen_range(0, list.len());
            return Ok(list[random_index].clone());
        }
    }

    fn Remove(&self, word: ByteString) -> Fallible<()> {
        let mut list = self.such_list.borrow_mut();
        let mut present = 0;
        for i in 0..list.len() {
                if word == list[i] {
                list.remove(i);
                present = 1;
            }
        }
        if present == 0 {
            return Err(Error::Type("The word doesn't exist in the list".to_string()));
        } else {
            return Ok(());
        }
    }
}