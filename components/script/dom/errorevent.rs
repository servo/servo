use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::InheritTypes::{EventCast, ErrorEventDerived};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::global;
use dom::bindings::js::{MutNullableJS, JSRef, RootedReference, Temporary, OptionalSettable};
use js::jsapi::JSContext;

use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::event::{Event, EventTypeId, ErrorEventTypeId	};
use dom::window::Window;
use servo_util::str::DOMString;

use std::cell::{Cell, RefCell};
use std::default::Default;
use js::jsval::{JSVal, NullValue};
use std::u32;

#[jstraceable]
#[must_root]
//#[dom_struct]
#[privatize]
pub struct ErrorEvent {		
    event: Event,
		message: RefCell<DOMString>,
		filename: RefCell<DOMString>,
		lineno: Cell<u32>,	
		colno: Cell<u32>,
		error: Cell<JSVal>
}

impl ErrorEventDerived for Event {
    fn is_errorevent(&self) -> bool {
        *self.type_id() == ErrorEventTypeId
    }
}

impl ErrorEvent {
    pub fn new_inherited(type_id: EventTypeId) -> ErrorEvent {
        ErrorEvent {	 
            event: Event::new_inherited(type_id),
			message: RefCell::new("".to_string()),
			filename: RefCell::new("".to_string()),
			lineno: Cell::new(0),
			colno: Cell::new(0),
			error: Cell::new(NullValue())
        }
    }

    pub fn new_uninitialized(window: JSRef<Window>) -> Temporary<ErrorEvent> {
        reflect_dom_object(box ErrorEvent::new_inherited(ErrorEventTypeId),
                           &global::Window(window),
                           ErrorEventBinding::Wrap)
    }

    pub fn new(window: JSRef<Window>,
							 global: &GlobalRef,
               type_: DOMString,
               can_bubble: bool,
               cancelable: bool,
	       			 message: DOMString,
	       			 filename: DOMString,
		 			 		 lineno: u32,
			   			 colno: u32,
               error: JSVal) -> Temporary<ErrorEvent> {	
        let ev = ErrorEvent::new_uninitialized(window).root();
        ev.InitErrorEvent(global.get_cx(),type_, can_bubble, cancelable, message, filename, lineno, colno, error);
        Temporary::from_rooted(*ev)
    }

    pub fn Constructor(global: &GlobalRef,
                       type_: DOMString,
                       init: &ErrorEventBinding::ErrorEventInit) -> Fallible<Temporary<ErrorEvent>> {
				//let input_num: Option<u32> = from_str(init.lineno);
				let msg = match init.message {
        Some(msg) => msg
    };
			
				let file_name = match init.filename {
        Some(file_name) => file_name
    };


				let line_num = match init.lineno {
        Some(line_num) => line_num
    };

				let col_num = match init.colno {
        Some(col_num) => col_num
    };

        let event = ErrorEvent::new(global.as_window(), global, type_,
                                 init.parent.bubbles, init.parent.cancelable,
                                 msg, file_name,
				 line_num, col_num, init.error);
        Ok(event)
    }

    #[inline]
    pub fn event<'a>(&'a self) -> &'a Event {
        &self.event
    }
}

impl<'a> ErrorEventMethods for JSRef<'a, ErrorEvent> {

	
		fn Lineno(self) -> u32 {
				self.lineno.get()
		}

		fn Colno(self) -> u32 {
				self.colno.get()
		}
    
		fn Message(self) -> DOMString {
				//let mut r = self.message.borrow_mut();
				//let data = (&mut r).get();
				self.message.borrow().clone();
		}

		/*fn SetMessage(self, m: DOMString ) {
				//self.message.get()
		} */

		fn Filename(self) -> DOMString {
				self.filename.borrow().clone();
		}

		fn Error(self) -> JSVal {
				self.error.get()
		}



    fn InitErrorEvent(self,
									 _cx: *mut JSContext,
                   type_: DOMString,
                   can_bubble: bool,
                   cancelable: bool,
									 message: DOMString,
									 filename: DOMString,
									 lineno: u32,
									 colno: u32,
                   error: JSVal) {
        let event: JSRef<Event> = EventCast::from_ref(self);
        event.InitEvent(type_, can_bubble, cancelable);
		*self.message.borrow_mut() = message;
		
		*self.filename.borrow_mut() = filename;
		self.lineno.set(lineno);
		self.colno.set(colno);
		self.error.set(error);
    }
}

impl Reflectable for ErrorEvent {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.event.reflector()
    }
}

