/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::refcounted::Trusted;

use crate::dom::bindings::root::Dom;
use crate::dom::document::document::Document;
use crate::dom::document::loading::pending_script::PendingInOrderScriptVec;
use crate::dom::html::htmlscriptelement::{HTMLScriptElement, ScriptResult};
use crate::dom::node::NodeTraits;

#[derive(Clone, Copy, Default, MallocSizeOf, PartialEq)]
pub(crate) enum TheEndLoadingPhase {
    #[default]
    Initial,
    ProcessingDeferredScripts,
    ProcessingAsSoonAsPossibleScripts,
    WaitingForLoadEventBlockers,
    Done,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentLoadingHandler {
    document: Dom<Document>,

    #[no_trace]
    current_the_end_loading_phase: Cell<TheEndLoadingPhase>,

    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing>
    deferred_scripts: PendingInOrderScriptVec,

    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible>
    asap_in_order_scripts_list: PendingInOrderScriptVec,

    /// <https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible>
    asap_scripts_set: DomRefCell<Vec<Dom<HTMLScriptElement>>>,
}

impl DocumentLoadingHandler {
    pub(crate) fn new(document: &Document) -> Self {
        Self {
            document: Dom::from_ref(document),
            current_the_end_loading_phase: Default::default(),
            deferred_scripts: Default::default(),
            asap_in_order_scripts_list: Default::default(),
            asap_scripts_set: Default::default(),
        }
    }

    pub(crate) fn current_the_end_loading_phase(&self) -> TheEndLoadingPhase {
        self.current_the_end_loading_phase.get()
    }

    // TODO(47965): Remove once refactoring is completed
    pub(crate) fn set_current_the_end_loading_phase(&self, loading_phase: TheEndLoadingPhase) {
        self.current_the_end_loading_phase.set(loading_phase);
    }

    pub(crate) fn start_the_end_loading_phase(&self) {
        self.current_the_end_loading_phase
            .set(TheEndLoadingPhase::ProcessingDeferredScripts);
    }

    /// <https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-when-the-document-has-finished-parsing>
    pub(crate) fn add_deferred_script(&self, script: &HTMLScriptElement) {
        self.deferred_scripts.push(script);
    }

    pub(crate) fn deferred_script_loaded(
        &self,
        cx: &mut JSContext,
        element: &HTMLScriptElement,
        result: ScriptResult,
    ) {
        self.deferred_scripts.loaded(element, result);
        self.process_deferred_scripts(cx);
    }

    // https://html.spec.whatwg.org/multipage/#set-of-scripts-that-will-execute-as-soon-as-possible
    pub(crate) fn add_asap_script(&self, script: &HTMLScriptElement) {
        self.asap_scripts_set
            .borrow_mut()
            .push(Dom::from_ref(script));
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 5.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step 22.d.
    pub(crate) fn asap_script_loaded(
        &self,
        cx: &mut JSContext,
        element: &HTMLScriptElement,
        result: ScriptResult,
    ) {
        {
            let mut scripts = self.asap_scripts_set.borrow_mut();
            let idx = scripts
                .iter()
                .position(|entry| &**entry == element)
                .unwrap();
            scripts.swap_remove(idx);
        }
        element.execute(cx, result);
        self.wait_until_asap_scripts_have_executed();
    }

    // https://html.spec.whatwg.org/multipage/#list-of-scripts-that-will-execute-in-order-as-soon-as-possible
    pub(crate) fn push_asap_in_order_script(&self, script: &HTMLScriptElement) {
        self.asap_in_order_scripts_list.push(script);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-end> step 5.
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script> step> 22.c.
    pub(crate) fn asap_in_order_script_loaded(
        &self,
        cx: &mut JSContext,
        element: &HTMLScriptElement,
        result: ScriptResult,
    ) {
        self.asap_in_order_scripts_list.loaded(element, result);
        while let Some((element, result)) = self
            .asap_in_order_scripts_list
            .take_next_ready_to_be_executed()
        {
            element.execute(cx, result);
        }

        self.wait_until_asap_scripts_have_executed();
    }

    fn has_finished_all_asap_scripts(&self) -> bool {
        self.asap_scripts_set.borrow().is_empty() && self.asap_in_order_scripts_list.is_empty()
    }

    /// Step 5 of <https://html.spec.whatwg.org/multipage/#the-end>
    pub(crate) fn process_deferred_scripts(&self, cx: &mut JSContext) {
        if self.current_the_end_loading_phase.get() != TheEndLoadingPhase::ProcessingDeferredScripts
        {
            return;
        }

        // Step 5.1. Spin the event loop until the first script in the list of scripts that will execute when the
        // document has finished parsing has its ready to be parser-executed set to true and the parser's Document
        // has no style sheet that is blocking scripts.
        loop {
            if self.document.has_a_stylesheet_that_is_blocking_scripts() {
                return;
            }
            // Step 5.3. Remove the first script element from the list of scripts that will execute when the
            // document has finished parsing (i.e. shift out the first entry in the list).
            if let Some((element, result)) = self.deferred_scripts.take_next_ready_to_be_executed()
            {
                // Step 5.2. Execute the script element given by the
                // first script in the list of scripts that will execute when the document has finished parsing.
                element.execute(cx, result);
            } else {
                break;
            }
        }
        // Step 5. While the list of scripts that will execute when the document has finished parsing is not empty:
        if self.deferred_scripts.is_empty() {
            self.current_the_end_loading_phase
                .set(TheEndLoadingPhase::ProcessingAsSoonAsPossibleScripts);
            self.document.dispatch_dom_content_loaded();
        }
    }

    /// Step 7 of <https://html.spec.whatwg.org/multipage/#the-end>
    pub(crate) fn wait_until_asap_scripts_have_executed(&self) {
        if self.current_the_end_loading_phase.get() !=
            TheEndLoadingPhase::ProcessingAsSoonAsPossibleScripts
        {
            return;
        }
        // Step 7. Spin the event loop until the set of scripts that will execute as soon as possible
        // and the list of scripts that will execute in order as soon as possible are empty.
        if self.has_finished_all_asap_scripts() {
            let document = Trusted::new(&*self.document);
            self.document
                .owner_global()
                .task_manager()
                .dom_manipulation_task_source()
                .queue(task!(transition_away_from_asap_scripts: move |cx| {
                    let document = document.root();
                    {
                        let Some(ref loading_handler) = *document.loading_handler() else {
                            return;
                        };
                        // Ensure that if this task is fired multiple times, we only progress the
                        // end of loading phase once.
                        if loading_handler.current_the_end_loading_phase.get() !=
                            TheEndLoadingPhase::ProcessingAsSoonAsPossibleScripts
                        {
                            return;
                        }
                        // Check again if we still fulfil the goal
                        if !loading_handler.has_finished_all_asap_scripts() {
                            return;
                        }
                        loading_handler.current_the_end_loading_phase
                                .set(TheEndLoadingPhase::WaitingForLoadEventBlockers);
                    }
                    document.wait_until_load_blockers_have_resolved(cx);
                }));
        }
    }
}
