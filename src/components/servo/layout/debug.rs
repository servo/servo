/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub trait BoxedMutDebugMethods {
    fn dump(@mut self);
    fn dump_indent(@mut self, ident: uint);
    fn debug_str(@mut self) -> ~str;
}

pub trait BoxedDebugMethods {
    fn dump(@self);
    fn dump_indent(@self, ident: uint);
    fn debug_str(@self) -> ~str;
}

pub trait DebugMethods {
    fn dump(&self);
    fn dump_indent(&self, ident: uint);
    fn debug_str(&self) -> ~str;
}
