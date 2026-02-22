/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;
use crate::dom::execcommand::basecommand::BaseCommand;
use crate::dom::execcommand::contenteditable::SelectionExecCommandSupport;
use crate::dom::selection::Selection;

pub(crate) struct DeleteCommand {}

impl BaseCommand for DeleteCommand {
    /// <https://w3c.github.io/editing/docs/execCommand/#the-delete-command>
    fn execute(&self, selection: &Selection, _value: DOMString) -> bool {
        let active_range = selection
            .active_range()
            .expect("Must always have an active range");
        // Step 1. If the active range is not collapsed, delete the selection and return true.
        if !active_range.collapsed() {
            selection.delete_the_selection(&active_range);
            return true;
        }
        // Step 2. Canonicalize whitespace at the active range's start.
        // TODO

        // Step 3. Let node and offset be the active range's start node and offset.
        // TODO

        // Step 4. Repeat the following steps:
        // TODO

        // Step 5. If node is a Text node and offset is not zero, or if node is
        // a block node that has a child with index offset − 1 and that child is a br or hr or img:
        // TODO

        // Step 6. If node is an inline node, return true.
        // TODO

        // Step 7. If node is an li or dt or dd and is the first child of its parent, and offset is zero:
        // TODO

        // Step 8. Let start node equal node and let start offset equal offset.
        // TODO

        // Step 9. Repeat the following steps:
        // TODO

        // Step 10. If offset is zero, and node has an editable inclusive ancestor in the same editing host that's an indentation element:
        // TODO

        // Step 11. If the child of start node with index start offset is a table, return true.
        // TODO

        // Step 12. If start node has a child with index start offset − 1, and that child is a table:
        // TODO

        // Step 13. If offset is zero; and either the child of start node with index start offset
        // minus one is an hr, or the child is a br whose previousSibling is either a br or not an inline node:
        // TODO

        // Step 14. If the child of start node with index start offset is an li or dt or dd, and
        // that child's firstChild is an inline node, and start offset is not zero:
        // TODO

        // Step 15. If start node's child with index start offset is an li or dt or dd, and
        // that child's previousSibling is also an li or dt or dd:
        // TODO

        // Step 16. While start node has a child with index start offset minus one:
        // TODO

        // Step 17. Call collapse(start node, start offset) on the context object's selection.
        // TODO

        // Step 18. Call extend(node, offset) on the context object's selection.
        // TODO

        // Step 19. Delete the selection, with direction "backward".
        // TODO

        // Step 20. Return true.
        true
    }
}
