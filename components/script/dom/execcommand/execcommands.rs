/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::selection::Selection;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/editing/docs/execCommand/#miscellaneous-commands>
fn is_command_listed_in_miscellaneous_section(command_name: CommandName) -> bool {
    matches!(
        command_name,
        CommandName::DefaultParagraphSeparator |
            CommandName::Redo |
            CommandName::SelectAll |
            CommandName::StyleWithCss |
            CommandName::Undo |
            CommandName::Usecss
    )
}

impl Document {
    /// <https://w3c.github.io/editing/docs/execCommand/#enabled>
    fn selection_if_command_is_enabled(
        &self,
        cx: &mut js::context::JSContext,
        command_name: CommandName,
    ) -> Option<DomRoot<Selection>> {
        let selection = self.GetSelection(CanGc::from_cx(cx))?;
        // > Among commands defined in this specification, those listed in Miscellaneous commands are always enabled,
        // > except for the cut command and the paste command.
        //
        // Note: cut and paste are listed in the "clipboard commands" section, not the miscellaneous section
        if is_command_listed_in_miscellaneous_section(command_name) {
            return Some(selection);
        }
        // > The other commands defined here are enabled if the active range is not null,
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
    fn command_if_command_is_supported(&self, command_id: &DOMString) -> Option<CommandName> {
        // https://w3c.github.io/editing/docs/execCommand/#methods-to-query-and-execute-commands
        // > All of these methods must treat their command argument ASCII case-insensitively.
        Some(match &*command_id.str().to_lowercase() {
            "delete" => CommandName::Delete,
            "defaultparagraphseparator" => CommandName::DefaultParagraphSeparator,
            "stylewithcss" => CommandName::StyleWithCss,
            _ => return None,
        })
    }
}

pub(crate) trait DocumentExecCommandSupport {
    fn is_command_supported(&self, command_id: DOMString) -> bool;
    fn is_command_indeterminate(&self, command_id: DOMString) -> bool;
    fn command_state_for_command(&self, command_id: DOMString) -> bool;
    fn command_value_for_command(&self, command_id: DOMString) -> DOMString;
    fn check_support_and_enabled(
        &self,
        cx: &mut js::context::JSContext,
        command_id: &DOMString,
    ) -> Option<(CommandName, DomRoot<Selection>)>;
    fn exec_command_for_command_id(
        &self,
        cx: &mut js::context::JSContext,
        command_id: DOMString,
        value: DOMString,
    ) -> bool;
}

impl DocumentExecCommandSupport for Document {
    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandsupported()>
    fn is_command_supported(&self, command_id: DOMString) -> bool {
        self.command_if_command_is_supported(&command_id).is_some()
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandindeterm()>
    fn is_command_indeterminate(&self, command_id: DOMString) -> bool {
        // Step 1. If command is not supported or has no indeterminacy, return false.
        // Step 2. Return true if command is indeterminate, otherwise false.
        self.command_if_command_is_supported(&command_id)
            .is_some_and(|command| command.is_indeterminate())
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandstate()>
    fn command_state_for_command(&self, command_id: DOMString) -> bool {
        // Step 1. If command is not supported or has no state, return false.
        let Some(command) = self.command_if_command_is_supported(&command_id) else {
            return false;
        };
        let Some(state) = command.current_state(self) else {
            return false;
        };
        // Step 2. If the state override for command is set, return it.
        // Step 3. Return true if command's state is true, otherwise false.
        self.state_override(&command).unwrap_or(state)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandvalue()>
    fn command_value_for_command(&self, command_id: DOMString) -> DOMString {
        // Step 1. If command is not supported or has no value, return the empty string.
        let Some(command) = self.command_if_command_is_supported(&command_id) else {
            return DOMString::new();
        };
        let Some(value) = command.current_value(self) else {
            return DOMString::new();
        };
        // Step 2. If command is "fontSize" and its value override is set,
        // convert the value override to an integer number of pixels and return the legacy font size for the result.
        // TODO

        // Step 3. If the value override for command is set, return it.
        if let Some(value_override) = self.value_override(&command) {
            return value_override;
        }
        // Step 4. Return command's value.
        value
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandenabled()>
    fn check_support_and_enabled(
        &self,
        cx: &mut js::context::JSContext,
        command_id: &DOMString,
    ) -> Option<(CommandName, DomRoot<Selection>)> {
        // Step 2. Return true if command is both supported and enabled, false otherwise.
        let command = self.command_if_command_is_supported(command_id)?;
        let selection = self.selection_if_command_is_enabled(cx, command)?;
        Some((command, selection))
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#execcommand()>
    fn exec_command_for_command_id(
        &self,
        cx: &mut js::context::JSContext,
        command_id: DOMString,
        value: DOMString,
    ) -> bool {
        // Step 3. If command is not supported or not enabled, return false.
        let Some((command, selection)) = self.check_support_and_enabled(cx, &command_id) else {
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
        let result = command.execute(cx, self, &selection, value);
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
