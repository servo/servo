/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::Window;

enum Validity {
	ValueMissing,
	TypeMismatch,
	PatternMismatch,
	TooLong,
	TooShort,
	RangeUnderflow,
	RangeOverflow,
	StepMismatch,
	BadInput,
	CustomError,
	Valid
}

// https://html.spec.whatwg.org/#validitystate
#[dom_struct(Element)]
pub struct ValidityState {
	reflector_: Reflector,
	state: u8,
}

impl ValidityState {
	fn new_inherited() -> ValidityState {
		ValidityState {
			reflector_: Reflector::new(),
			state: 0,
		}
	}

	pub fn new(window: &Window) -> Root<ValidityState> {
		reflect_dom_object(box ValidityState::new_inherited(),
				GlobalRef::Window(window),
				ValidityStateBinding::Wrap)
	}
}

impl ValidityStateMethods for ValidityState {
	fn ValueMissing(&self) -> bool {
		return false;
	}

	fn TypeMismatch(&self) -> bool {
		return false;
	}

	fn PatternMismatch(&self) -> bool {
		return false;
	}

	fn TooLong(&self) -> bool {
		return false;
	}

	fn TooShort(&self) -> bool {
		return false;
	}

	fn RangeUnderflow(&self) -> bool {
		return false;
	}

	fn RangeOverflow(&self) -> bool {
		return false;
	}

	fn StepMismatch(&self) -> bool {
		return false;
	}

	fn BadInput(&self) -> bool {
		return false;
	}

	fn CustomError(&self) -> bool {
		return false;
	}

	fn Valid(&self) -> bool {
		return false;
	}
}
