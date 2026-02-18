/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::BaseCommand;
use crate::dom::execcommand::commands::delete::DeleteCommand;
use crate::dom::selection::Selection;
use crate::script_runtime::CanGc;

impl Document {
    /// <https://w3c.github.io/editing/docs/execCommand/#enabled>
    fn selection_if_command_is_enabled(&self, can_gc: CanGc) -> Option<DomRoot<Selection>> {
        // > Among commands defined in this specification, those listed in Miscellaneous commands are always enabled,
        // > except for the cut command and the paste command.
        // TODO
        // > The other commands defined here are enabled if the active range is not null,
        let selection = self.GetSelection(can_gc)?;
        let range = selection.active_range()?;
        // > its start node is either editable or an editing host,
        if !range.start_container().is_editable_or_editing_host() {
            return None;
        }
        // > the editing host of its start node is not an EditContext editing host,
        // TODO
        // > its end node is either editable or an editing host,
        if !range.end_container().is_editable_or_editing_host() {
            return None;
        }
        // > the editing host of its end node is not an EditContext editing host,
        // TODO
        // > and there is some editing host that is an inclusive ancestor of both its start node and its end node.
        // TODO
        Some(selection)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#supported>
    fn command_if_command_is_supported(
        &self,
        command_id: DOMString,
    ) -> Option<Box<dyn BaseCommand>> {
        Some(Box::new(match &*command_id.str() {
            "delete" => DeleteCommand {},
            _ => return None,
        }))
    }
}

pub(crate) trait ExecCommandsSupport {
    fn check_support_and_enabled(
        &self,
        command_id: DOMString,
        can_gc: CanGc,
    ) -> Option<(Box<dyn BaseCommand>, DomRoot<Selection>)>;
    fn exec_command_for_command_id(
        &self,
        command_id: DOMString,
        value: DOMString,
        can_gc: CanGc,
    ) -> bool;
}

impl ExecCommandsSupport for Document {
    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandenabled()>
    fn check_support_and_enabled(
        &self,
        command_id: DOMString,
        can_gc: CanGc,
    ) -> Option<(Box<dyn BaseCommand>, DomRoot<Selection>)> {
        // Step 2. Return true if command is both supported and enabled, false otherwise.
        self.command_if_command_is_supported(command_id)
            .zip(self.selection_if_command_is_enabled(can_gc))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#execcommand()>
    fn exec_command_for_command_id(
        &self,
        command_id: DOMString,
        value: DOMString,
        can_gc: CanGc,
    ) -> bool {
        // Step 3. If command is not supported or not enabled, return false.
        let Some((command, selection)) = self.check_support_and_enabled(command_id, can_gc) else {
            return false;
        };
        // Step 4. If command is not in the Miscellaneous commands section:
        // TODO

        // Step 4.1. Let affected editing host be the editing host that is an inclusive ancestor
        // of the active range's start node and end node, and is not the ancestor of any editing host
        // that is an inclusive ancestor of the active range's start node and end node.
        // TODO

        // Step 4.2. Fire an event named "beforeinput" at affected editing host using InputEvent,
        // with its bubbles and cancelable attributes initialized to true, and its data attribute initialized to null
        // TODO

        // Step 4.3. If the value returned by the previous step is false, return false.
        // TODO

        // Step 4.4. If command is not enabled, return false.
        // TODO

        // Step 4.5. Let affected editing host be the editing host that is an inclusive ancestor
        // of the active range's start node and end node, and is not the ancestor of any editing host
        // that is an inclusive ancestor of the active range's start node and end node.
        // TODO

        // Step 5. Take the action for command, passing value to the instructions as an argument.
        let result = command.execute(&selection, value);
        // Step 6. If the previous step returned false, return false.
        if !result {
            return false;
        }
        // Step 7. If the action modified DOM tree, then fire an event named "input" at affected editing
        // host using InputEvent, with its isTrusted and bubbles attributes initialized to true,
        // inputType attribute initialized to the mapped value of command, and its data attribute initialized to null.
        // TODO

        // Step 8. Return true.
        true
    }
}
