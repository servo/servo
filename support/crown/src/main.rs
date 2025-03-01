/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(rustc_private)]
#![warn(rustc::internal)]
#![allow(rustc::potential_query_instability)]

// This rustc crates are private so they must be manually imported.
extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_log;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;
extern crate rustc_type_ir;

use std::path::Path;

use rustc_driver::Callbacks;
use rustc_interface::interface::Config;

mod common;

#[cfg(feature = "unrooted_must_root_lint")]
mod unrooted_must_root;

#[cfg(feature = "trace_in_no_trace_lint")]
mod trace_in_no_trace;

mod crown_is_not_used;

struct MyCallbacks;

impl Callbacks for MyCallbacks {
    fn config(&mut self, config: &mut Config) {
        config.register_lints = Some(Box::new(move |sess, lint_store| {
            // Skip checks for proc-macro crates.
            if sess
                .opts
                .crate_types
                .contains(&rustc_session::config::CrateType::ProcMacro)
            {
                return;
            }

            crown_is_not_used::register(lint_store);
            #[cfg(feature = "unrooted_must_root_lint")]
            unrooted_must_root::register(lint_store);
            #[cfg(feature = "trace_in_no_trace_lint")]
            trace_in_no_trace::register(lint_store);
        }));
    }
}

fn main() {
    let handler =
        rustc_session::EarlyDiagCtxt::new(rustc_session::config::ErrorOutputType::default());
    rustc_driver::init_logger(&handler, rustc_log::LoggerConfig::from_env("CROWN_LOG"));
    let mut args: Vec<_> = std::env::args().collect();

    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this
    if args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref()) {
        args.remove(1);
    }

    // Pass cfg(crown) to rustc
    args.extend(["--cfg".to_owned(), "crown".to_owned()]);

    rustc_driver::RunCompiler::new(&args, &mut MyCallbacks).run()
}
