use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::CloseEventBinding;
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::InheritTypes::EventCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalField, GlobalRef, GlobalRoot};
use dom::bindings::js::{JSRef,Temporary};
use dom::event::{Event, EventTypeId, EventBubbles, EventCancelable};
use script_task::{ScriptChan, ScriptMsg, Runnable};

use dom::bindings::utils::reflect_dom_object;
use std::borrow::ToOwned;
use dom::bindings::cell::DOMRefCell;
use std::cell::Cell;
use util::str::DOMString;


#[dom_struct]
pub struct CloseEvent{
	event: Event,
	wasClean: Cell<bool>,
	code: Cell<u16>,
	reason: DOMRefCell<DOMString>
}

#[derive(PartialEq)]
pub enum Clean {
    Clean,
    NotClean
}

impl CloseEvent{
	pub fn new_inherited(type_id: EventTypeId) -> CloseEvent{
		println!("Creating CloseEvent..");
		CloseEvent{
			event: Event::new_inherited(type_id),
			wasClean: Cell::new(false),
			code: Cell::new(0),
			reason: DOMRefCell::new("".to_owned())
		}
	}

	pub fn new_uninitialized(global: GlobalRef) -> Temporary<CloseEvent> {
		reflect_dom_object(box CloseEvent::new_inherited(EventTypeId::CloseEvent),
				   global,
				   CloseEventBinding::Wrap)
	}

	pub fn new(global: GlobalRef,
		   type_: DOMString,
		   bubbles: EventBubbles,
		   cancelable: EventCancelable,
		   wasClean: bool,
		   code: u16,
		   reason: DOMString) -> Temporary<CloseEvent> {
		let ev = CloseEvent::new_uninitialized(global).root();
		let event: JSRef<Event> = EventCast::from_ref(ev.r());
        	event.InitEvent(type_,
				bubbles == EventBubbles::Bubbles,
				cancelable == EventCancelable::Cancelable);
		let ev = ev.r();
		ev.wasClean.set(wasClean);
		ev.code.set(code);
		*ev.reason.borrow_mut() = reason;
	        Temporary::from_rooted(ev)
	}

	pub fn Constructor(global: GlobalRef,
			   type_: DOMString,
			   init: &CloseEventBinding::CloseEventInit) -> Fallible<Temporary<CloseEvent>> {
		let clean_status = init.wasClean.unwrap();
		let cd = init.code.unwrap_or(0);
		let rsn = match init.reason.as_ref() {
			Some(reason) => reason.clone(),
			None => "".to_owned(),
		};
        	let bubbles = if init.parent.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
	        let cancelable = if init.parent.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
    		Ok(CloseEvent::new(global, type_, bubbles, cancelable, clean_status, cd, rsn))
	}
}

impl<'a> CloseEventMethods for JSRef<'a, CloseEvent>{
	fn WasClean(self) -> bool {
		self.wasClean.get()
	}

	fn Code(self) -> u16 {
		self.code.get()
	}

	fn Reason(self) -> DOMString {
		let reason = self.reason.borrow();
		reason.clone()
	}
}
