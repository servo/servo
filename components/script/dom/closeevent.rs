
use dom::bindings::codegen::Bindings::CloseEventBinding::CloseEventMethods;
use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalField, GlobalRef, GlobalRoot};
use dom::bindings::js::{JSRef,Temporary};
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use script_task::{ScriptChan, ScriptMsg, Runnable};
use std::borrow::ToOwned;
use std::cell::Cell;
use util::str::DOMString;


#[dom_struct]
pub struct CloseEvent{
	wasClean: Cell<bool>,
	code: Cell<u16>,
	reason: Cell<DOMString>
}

impl CloseEvent{
	pub fn new_inherited(global: GlobalRef,
			     clean: bool,
			     cd: u16,
			     rsn: DOMString) -> Event{
		println!("Creating CloseEvent..");
		CloseEvent{
			wasClean: Cell::new(clean),
			code: Cell::new(cd),
			reason: Cell::new(rsn.to_owned())
		}
	}

	pub fn new(global: GlobalRef,
		   type_: DOMString,
		   bubbles: EventBubbles,
		   cancelable: EventCancelable) -> Temporary<Event> {
		let ce_root = Event::new_uninitialized(global).root();
        	ce_root.r().InitEvent(type_,
				      bubbles == EventBubbles::Bubbles,
				      cancelable == EventCancelable::Cancelable
				     );
	        Temporary::from_rooted(ce_root.r())
	}

	pub fn Constructor(global: GlobalRef,
			   type_: DOMString,
			   init: &EventBinding::EventInit) -> Fallible<Temporary<Event>> {
        	let bubbles = if init.bubbles { EventBubbles::Bubbles } else { EventBubbles::DoesNotBubble };
	        let cancelable = if init.cancelable { EventCancelable::Cancelable } else { EventCancelable::NotCancelable };
    		Ok(CloseEvent::new(global, type_, bubbles, cancelable))
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
		self.get()
	}
}
