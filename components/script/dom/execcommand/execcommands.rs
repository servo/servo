/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::event::Event;
use crate::dom::event::inputevent::InputEvent;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::execcommand::commands::fontsize::maybe_normalize_pixels;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::Node;
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

/// <https://w3c.github.io/editing/docs/execCommand/#dfn-map-an-edit-command-to-input-type-value>
fn mapped_value_of_command(command: CommandName) -> DOMString {
    match command {
        CommandName::BackColor => "formatBackColor",
        CommandName::Bold => "formatBold",
        CommandName::CreateLink => "insertLink",
        CommandName::Cut => "deleteByCut",
        CommandName::Delete => "deleteContentBackward",
        CommandName::FontName => "formatFontName",
        CommandName::ForeColor => "formatFontColor",
        CommandName::ForwardDelete => "deleteContentForward",
        CommandName::Indent => "formatIndent",
        CommandName::InsertHorizontalRule => "insertHorizontalRule",
        CommandName::InsertLineBreak => "insertLineBreak",
        CommandName::InsertOrderedList => "insertOrderedList",
        CommandName::InsertParagraph => "insertParagraph",
        CommandName::InsertText => "insertText",
        CommandName::InsertUnorderedList => "insertUnorderedList",
        CommandName::JustifyCenter => "formatJustifyCenter",
        CommandName::JustifyFull => "formatJustifyFull",
        CommandName::JustifyLeft => "formatJustifyLeft",
        CommandName::JustifyRight => "formatJustifyRight",
        CommandName::Outdent => "formatOutdent",
        CommandName::Paste => "insertFromPaste",
        CommandName::Redo => "historyRedo",
        CommandName::Strikethrough => "formatStrikeThrough",
        CommandName::Superscript => "formatSuperscript",
        CommandName::Undo => "historyUndo",
        _ => "",
    }
    .into()
}

impl Node {
    fn is_in_plaintext_only_state(&self) -> bool {
        self.downcast::<HTMLElement>()
            .is_some_and(|el| el.ContentEditable().str() == "plaintext-only")
    }
}

impl Document {
    /// <https://w3c.github.io/editing/docs/execCommand/#enabled>
    fn selection_if_command_is_enabled(
        &self,
        cx: &mut JSContext,
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
        let start_container_editing_host = range.start_container().editing_host_of()?;
        // > the editing host of its start node is not an EditContext editing host,
        // TODO
        // > its end node is either editable or an editing host,
        let end_container_editing_host = range.end_container().editing_host_of()?;
        // > the editing host of its end node is not an EditContext editing host,
        // TODO
        // > and there is some editing host that is an inclusive ancestor of both its start node and its end node.
        // TODO

        // Some commands are only enabled if the editing host is *not* in plaintext-only state.
        if !command_name.is_enabled_in_plaintext_only_state() &&
            (start_container_editing_host.is_in_plaintext_only_state() ||
                end_container_editing_host.is_in_plaintext_only_state())
        {
            None
        } else {
            Some(selection)
        }
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#supported>
    fn command_if_command_is_supported(&self, command_id: &DOMString) -> Option<CommandName> {
        // https://w3c.github.io/editing/docs/execCommand/#methods-to-query-and-execute-commands
        // > All of these methods must treat their command argument ASCII case-insensitively.
        Some(match &*command_id.str().to_lowercase() {
            "delete" => CommandName::Delete,
            "defaultparagraphseparator" => CommandName::DefaultParagraphSeparator,
            "fontsize" => CommandName::FontSize,
            "stylewithcss" => CommandName::StyleWithCss,
            "underline" => CommandName::Underline,
            _ => return None,
        })
    }
}

pub(crate) trait DocumentExecCommandSupport {
    fn is_command_supported(&self, command_id: DOMString) -> bool;
    fn is_command_indeterminate(&self, command_id: DOMString) -> bool;
    fn command_state_for_command(&self, cx: &mut JSContext, command_id: DOMString) -> bool;
    fn command_value_for_command(&self, cx: &mut JSContext, command_id: DOMString) -> DOMString;
    fn check_support_and_enabled(
        &self,
        cx: &mut JSContext,
        command_id: &DOMString,
    ) -> Option<(CommandName, DomRoot<Selection>)>;
    fn exec_command_for_command_id(
        &self,
        cx: &mut JSContext,
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
    fn command_state_for_command(&self, cx: &mut JSContext, command_id: DOMString) -> bool {
        // Step 1. If command is not supported or has no state, return false.
        let Some(command) = self.command_if_command_is_supported(&command_id) else {
            return false;
        };
        let Some(state) = command.current_state(cx, self) else {
            return false;
        };
        // Step 2. If the state override for command is set, return it.
        // Step 3. Return true if command's state is true, otherwise false.
        self.state_override(&command).unwrap_or(state)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandvalue()>
    fn command_value_for_command(&self, cx: &mut JSContext, command_id: DOMString) -> DOMString {
        // Step 1. If command is not supported or has no value, return the empty string.
        let Some(command) = self.command_if_command_is_supported(&command_id) else {
            return DOMString::new();
        };
        let Some(value) = command.current_value(cx, self) else {
            return DOMString::new();
        };
        // Step 3. If the value override for command is set, return it.
        self.value_override(&command)
            .map(|value_override| {
                // Step 2. If command is "fontSize" and its value override is set,
                // convert the value override to an integer number of pixels and return the legacy font size for the result.
                if command == CommandName::FontSize {
                    maybe_normalize_pixels(&value_override, self).unwrap_or(value_override)
                } else {
                    value_override
                }
            })
            // Step 4. Return command's value.
            .unwrap_or(value)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#querycommandenabled()>
    fn check_support_and_enabled(
        &self,
        cx: &mut JSContext,
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
        cx: &mut JSContext,
        command_id: DOMString,
        value: DOMString,
    ) -> bool {
        let window = self.window();
        // Step 3. If command is not supported or not enabled, return false.
        let Some((command, mut selection)) = self.check_support_and_enabled(cx, &command_id) else {
            return false;
        };
        // Step 4. If command is not in the Miscellaneous commands section:
        let affected_editing_host = if !is_command_listed_in_miscellaneous_section(command) {
            // Step 4.1. Let affected editing host be the editing host that is an inclusive ancestor
            // of the active range's start node and end node, and is not the ancestor of any editing host
            // that is an inclusive ancestor of the active range's start node and end node.
            let affected_editing_host = selection
                .active_range()
                .expect("Must always have an active range")
                .CommonAncestorContainer()
                .editing_host_of()
                .expect("Must always have an editing host if command is enabled");

            // Step 4.2. Fire an event named "beforeinput" at affected editing host using InputEvent,
            // with its bubbles and cancelable attributes initialized to true, and its data attribute initialized to null
            let event = InputEvent::new(
                window,
                None,
                atom!("beforeinput"),
                true,
                true,
                Some(window),
                0,
                None,
                false,
                "".into(),
                CanGc::from_cx(cx),
            );
            let event = event.upcast::<Event>();
            // Step 4.3. If the value returned by the previous step is false, return false.
            if !event.fire(affected_editing_host.upcast(), CanGc::from_cx(cx)) {
                return false;
            }

            // Step 4.4. If command is not enabled, return false.
            let Some(new_selection) = self.selection_if_command_is_enabled(cx, command) else {
                return false;
            };
            selection = new_selection;

            // Step 4.5. Let affected editing host be the editing host that is an inclusive ancestor
            // of the active range's start node and end node, and is not the ancestor of any editing host
            // that is an inclusive ancestor of the active range's start node and end node.
            selection
                .active_range()
                .expect("Must always have an active range")
                .CommonAncestorContainer()
                .editing_host_of()
        } else {
            None
        };

        // Step 5. Take the action for command, passing value to the instructions as an argument.
        let result = command.execute(cx, self, &selection, value);
        // Step 6. If the previous step returned false, return false.
        if !result {
            return false;
        }
        // Step 7. If the action modified DOM tree, then fire an event named "input" at affected editing
        // host using InputEvent, with its isTrusted and bubbles attributes initialized to true,
        // inputType attribute initialized to the mapped value of command, and its data attribute initialized to null.
        if let Some(affected_editing_host) = affected_editing_host {
            let event = InputEvent::new(
                window,
                None,
                atom!("input"),
                true,
                false,
                Some(window),
                0,
                None,
                false,
                mapped_value_of_command(command),
                CanGc::from_cx(cx),
            );
            let event = event.upcast::<Event>();
            event.set_trusted(true);
            event.fire(affected_editing_host.upcast(), CanGc::from_cx(cx));
        }

        // Step 8. Return true.
        true
    }
}
