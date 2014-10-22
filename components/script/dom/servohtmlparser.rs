/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The bulk of the HTML parser integration is in `script::parse::html`.
//! This module is mostly about its interaction with DOM memory management.

use dom::bindings::codegen::Bindings::ServoHTMLParserBinding;
use dom::bindings::global;
use dom::bindings::trace::JSTraceable;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::TrustedNodeAddress;
use dom::document::Document;
use parse::html::JSMessage;

use std::default::Default;
use std::cell::{Cell, UnsafeCell};
use std::kinds::marker;
use url::Url;
use js::jsapi::JSTracer;
use html5ever::tokenizer;
use html5ever::tree_builder;
use html5ever::tree_builder::{TreeBuilder, TreeBuilderOpts};

#[must_root]
#[jstraceable]
pub struct Sink {
    pub js_chan: Sender<JSMessage>,
    pub base_url: Option<Url>,
    pub document: JS<Document>,
}

pub type Tokenizer = tokenizer::Tokenizer<TreeBuilder<TrustedNodeAddress, Sink>>;

/// Like `RefCell<Tokenizer>`, but lets us break the rules when we need to,
/// for garbage collector integration.  See rust-lang/rust#18131.
pub struct TokenizerCell {
    value: UnsafeCell<Tokenizer>,
    borrowed: Cell<bool>,
    nocopy: marker::NoCopy,
    nosync: marker::NoSync,
}

pub struct TokenizerRefMut<'a> {
    _parent: &'a TokenizerCell,
}

impl TokenizerCell {
    fn new(value: Tokenizer) -> TokenizerCell {
        TokenizerCell {
            value: UnsafeCell::new(value),
            borrowed: Cell::new(false),
            nocopy: marker::NoCopy,
            nosync: marker::NoSync,
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> TokenizerRefMut<'a> {
        if self.borrowed.get() {
            fail!("TokenizerCell already borrowed!");
        }
        self.borrowed.set(true);
        TokenizerRefMut {
            _parent: self,
        }
    }
}

#[unsafe_destructor]  // because we have lifetime parameters
impl<'a> Drop for TokenizerRefMut<'a> {
    fn drop(&mut self) {
        debug_assert!(self._parent.borrowed.get());
        self._parent.borrowed.set(false);
    }
}

impl<'b> Deref<Tokenizer> for TokenizerRefMut<'b> {
    #[inline]
    fn deref<'a>(&'a self) -> &'a Tokenizer {
        unsafe {
            &*self._parent.value.get()
        }
    }
}

impl<'b> DerefMut<Tokenizer> for TokenizerRefMut<'b> {
    #[inline]
    fn deref_mut<'a>(&'a mut self) -> &'a mut Tokenizer {
        unsafe {
            &mut *self._parent.value.get()
        }
    }
}

// NB: JSTraceable is *not* auto-derived.
// You must edit the impl below if you add fields!
#[must_root]
#[privatize]
pub struct ServoHTMLParser {
    reflector_: Reflector,
    tokenizer: TokenizerCell,
}

impl ServoHTMLParser {
    #[allow(unrooted_must_root)]
    pub fn new(js_chan: Sender<JSMessage>, base_url: Option<Url>, document: JSRef<Document>)
            -> Temporary<ServoHTMLParser> {
        let window = document.window().root();
        let sink = Sink {
            js_chan: js_chan,
            base_url: base_url,
            document: JS::from_rooted(document),
        };

        let tb = TreeBuilder::new(sink, TreeBuilderOpts {
            ignore_missing_rules: true,
            .. Default::default()
        });

        let tok = tokenizer::Tokenizer::new(tb, Default::default());

        let parser = ServoHTMLParser {
            reflector_: Reflector::new(),
            tokenizer: TokenizerCell::new(tok),
        };

        reflect_dom_object(box parser, &global::Window(*window), ServoHTMLParserBinding::Wrap)
    }

    #[inline]
    pub fn tokenizer<'a>(&'a self) -> &'a TokenizerCell {
        &self.tokenizer
    }
}

impl Reflectable for ServoHTMLParser {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

struct Tracer {
    trc: *mut JSTracer,
}

impl tree_builder::Tracer<TrustedNodeAddress> for Tracer {
    fn trace_handle(&self, node: TrustedNodeAddress) {
        node.trace(self.trc);
    }
}

impl JSTraceable for ServoHTMLParser {
    fn trace(&self, trc: *mut JSTracer) {
        self.reflector_.trace(trc);

        let tracer = Tracer {
            trc: trc,
        };
        let tracer = &tracer as &tree_builder::Tracer<TrustedNodeAddress>;

        unsafe {
            // There might be an active TokenizerRefMut, especially if parsing
            // triggered a garbage collection.  It's safe to ignore the borrow
            // flag, because that mutable reference won't be used along the
            // code paths reachable from tracing.

            let tokenizer: &Tokenizer = &*self.tokenizer.value.get();
            let tree_builder = tokenizer.sink();
            tree_builder.trace_handles(tracer);
            tree_builder.sink().trace(trc);
        }
    }
}
