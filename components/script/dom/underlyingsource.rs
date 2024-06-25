/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use js::jsapi::{HandleValue, HandleValueArray, Heap};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::panic::maybe_resume_unwind;

use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::{
    ReadableStreamController, UnderlyingSourceCancelCallback, UnderlyingSourcePullCallback,
    UnderlyingSourceStartCallback,
};
use crate::dom::bindings::import::base::*;
use crate::dom::bindings::import::module::{CallSetup, Fallible};
use crate::dom::promise::Promise;
use crate::dom::types::GlobalScope;

impl UnderlyingSourcePullCallback {
    #[allow(unsafe_code)]
    pub fn call_with_existing_obj(
        &self,
        this_obj: &Heap<*mut JSObject>,
        controller: ReadableStreamController,
        exception_handling: ExceptionHandling,
    ) -> Fallible<Rc<Promise>> {
        let s = CallSetup::new(self, exception_handling);
        let cx = s.get_context();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        rooted_vec!(let mut argv);
        argv.extend((0..1).map(|_| Heap::default()));

        let argc = 1;

        rooted!(in(*cx) let mut argv_root = UndefinedValue());
        unsafe {
            (controller).to_jsval(*cx, argv_root.handle_mut());
        }
        {
            let arg = &mut argv[0];
            *arg = Heap::default();
            arg.set(argv_root.get());
        }

        rooted!(in(*cx) let callable = ObjectValue(self.callback()));
        rooted!(in(*cx) let rooted_this = this_obj.get());
        let ok = unsafe {
            JS_CallFunctionValue(
                *cx,
                rooted_this.handle(),
                callable.handle(),
                &HandleValueArray {
                    length_: argc as ::libc::size_t,
                    elements_: argv.as_ptr() as *const JSVal,
                },
                rval.handle_mut(),
            )
        };
        maybe_resume_unwind();
        if !ok {
            return Err(JSFailed);
        }
        let rval_decl: Rc<Promise> = unsafe {
            // Scope for our JSAutoRealm.

            rooted!(in(*cx) let global_obj = CurrentGlobalOrNull(*cx));
            let promise_global =
                GlobalScope::from_object_maybe_wrapped(global_obj.handle().get(), *cx);

            rooted!(in(*cx) let mut value_to_resolve = rval.handle().get());
            if !JS_WrapValue(*cx, value_to_resolve.handle_mut()) {
                return Err(JSFailed);
            }
            match Promise::new_resolved(&promise_global, cx, value_to_resolve.handle()) {
                Ok(value) => value,
                Err(error) => {
                    throw_dom_exception(cx, &promise_global, error);
                    return Err(JSFailed);
                },
            }
        };
        Ok(rval_decl)
    }
}

impl UnderlyingSourceStartCallback {
    // FIXME Is this a good way to do? What are other way to call with existing js object?
    #[allow(unsafe_code)]
    pub fn call_with_existing_obj(
        &self,
        this_obj: &Heap<*mut JSObject>,
        controller: ReadableStreamController,
        exception_handling: ExceptionHandling,
    ) -> Fallible<JSVal> {
        let s = CallSetup::new(self, exception_handling);
        let cx = s.get_context();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        rooted_vec!(let mut argv);
        argv.extend((0..1).map(|_| Heap::default()));

        let argc = 1;

        rooted!(in(*cx) let mut argv_root = UndefinedValue());
        unsafe {
            (controller).to_jsval(*cx, argv_root.handle_mut());
        }
        {
            let arg = &mut argv[0];
            *arg = Heap::default();
            arg.set(argv_root.get());
        }

        rooted!(in(*cx) let callable = ObjectValue(self.callback()));
        rooted!(in(*cx) let rooted_this = this_obj.get());
        let ok = unsafe {
            JS_CallFunctionValue(
                *cx,
                rooted_this.handle(),
                callable.handle(),
                &HandleValueArray {
                    length_: argc as ::libc::size_t,
                    elements_: argv.as_ptr() as *const JSVal,
                },
                rval.handle_mut(),
            )
        };
        maybe_resume_unwind();
        if !ok {
            return Err(JSFailed);
        }
        let rval_decl = rval.handle();
        Ok(rval_decl.get())
    }
}

impl UnderlyingSourceCancelCallback {
    #[allow(unsafe_code)]
    pub fn call_with_existing_obj(
        &self,
        this_obj: &Heap<*mut JSObject>,
        reason: Option<HandleValue>,
        exception_handling: ExceptionHandling,
    ) -> Fallible<Rc<Promise>> {
        let s = CallSetup::new(self, exception_handling);
        let cx = s.get_context();
        rooted!(in(*cx) let mut rval = UndefinedValue());
        rooted_vec!(let mut argv);
        argv.extend((0..1).map(|_| Heap::default()));

        let mut argc = 1;

        if reason.is_some() {
            rooted!(in(*cx) let mut argv_root = UndefinedValue());
            unsafe {
                (reason.unwrap()).to_jsval(*cx, argv_root.handle_mut());
            }
            {
                let arg = &mut argv[0];
                *arg = Heap::default();
                arg.set(argv_root.get());
            }
        } else if argc == 1 {
            // This is our current trailing argument; reduce argc
            argc -= 1;
        } else {
            argv[0] = Heap::default();
        }

        rooted!(in(*cx) let callable = ObjectValue(self.callback()));
        rooted!(in(*cx) let rooted_this = this_obj.get());
        let ok = unsafe {
            JS_CallFunctionValue(
                *cx,
                rooted_this.handle(),
                callable.handle(),
                &HandleValueArray {
                    length_: argc as ::libc::size_t,
                    elements_: argv.as_ptr() as *const JSVal,
                },
                rval.handle_mut(),
            )
        };
        maybe_resume_unwind();
        if !ok {
            return Err(JSFailed);
        }
        let rval_decl: Rc<Promise> = unsafe {
            // Scope for our JSAutoRealm.

            rooted!(in(*cx) let global_obj = CurrentGlobalOrNull(*cx));
            let promise_global =
                GlobalScope::from_object_maybe_wrapped(global_obj.handle().get(), *cx);

            rooted!(in(*cx) let mut value_to_resolve = rval.handle().get());
            if !JS_WrapValue(*cx, value_to_resolve.handle_mut()) {
                return Err(JSFailed);
            }
            match Promise::new_resolved(&promise_global, cx, value_to_resolve.handle()) {
                Ok(value) => value,
                Err(error) => {
                    throw_dom_exception(cx, &promise_global, error);
                    return Err(JSFailed);
                },
            }
        };
        Ok(rval_decl)
    }
}
