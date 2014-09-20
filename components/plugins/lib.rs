/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(macro_rules, plugin_registrar, quote, phase)]

#![deny(unused_imports, unused_variable)]



#[phase(plugin,link)]
extern crate syntax;
#[phase(plugin, link)]
extern crate rustc;
#[cfg(test)]
extern crate sync;

use rustc::lint::LintPassObject;
use rustc::plugin::Registry;

mod lints;
mod macros;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_lint_pass(box lints::TransmutePass as LintPassObject);
    reg.register_lint_pass(box lints::UnrootedPass as LintPassObject);
}

