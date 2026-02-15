/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::RangeBinding::RangeMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::range::Range;
use crate::dom::selection::Selection;

pub(crate) trait BaseCommand {
    fn execute(&self, selection: &Selection, value: DOMString) -> bool;

    /// <https://w3c.github.io/editing/docs/execCommand/#delete-the-selection>
    fn delete_the_selection(&self, _selection: &Selection, active_range: &Range) {
        // Step 1. If the active range is null, abort these steps and do nothing.
        //
        // Always passed in as argument

        // Step 2. Canonicalize whitespace at the active range's start.
        // TODO

        // Step 3. Canonicalize whitespace at the active range's end.
        // TODO

        // Step 4. Let (start node, start offset) be the last equivalent point for the active range's start.
        // TODO

        // Step 5. Let (end node, end offset) be the first equivalent point for the active range's end.
        // TODO

        // Step 6. If (end node, end offset) is not after (start node, start offset):
        // TODO

        // Step 7. If start node is a Text node and start offset is 0, set start offset to the index of start node,
        // then set start node to its parent.
        // TODO

        // Step 8. If end node is a Text node and end offset is its length, set end offset to one plus the index of end node,
        // then set end node to its parent.
        // TODO

        // Step 9. Call collapse(start node, start offset) on the context object's selection.
        // TODO

        // Step 10. Call extend(end node, end offset) on the context object's selection.
        // TODO

        // Step 11.
        //
        // This step does not exist in the spec

        // Step 12. Let start block be the active range's start node.
        // TODO

        // Step 13. While start block's parent is in the same editing host and start block is an inline node, set start block to its parent.
        // TODO

        // Step 14. If start block is neither a block node nor an editing host, or "span" is not an allowed child of start block,
        // or start block is a td or th, set start block to null.
        // TODO

        // Step 15. Let end block be the active range's end node.
        // TODO

        // Step 16. While end block's parent is in the same editing host and end block is an inline node, set end block to its parent.
        // TODO

        // Step 17. If end block is neither a block node nor an editing host, or "span" is not an allowed child of end block,
        // or end block is a td or th, set end block to null.
        // TODO

        // Step 18.
        //
        // This step does not exist in the spec

        // Step 19. Record current states and values, and let overrides be the result.
        // TODO

        // Step 20.
        //
        // This step does not exist in the spec

        // Step 21. If start node and end node are the same, and start node is an editable Text node:
        //
        // As per the spec:
        // > NOTE: This whole piece of the algorithm is based on deleteContents() in DOM Range, copy-pasted and then adjusted to fit.
        let _ = active_range.DeleteContents();

        // Step 22. If start node is an editable Text node, call deleteData() on it, with start offset as
        // the first argument and (length of start node − start offset) as the second argument.
        // TODO

        // Step 23. Let node list be a list of nodes, initially empty.
        // TODO

        // Step 24. For each node contained in the active range, append node to node list if the
        // last member of node list (if any) is not an ancestor of node; node is editable;
        // and node is not a thead, tbody, tfoot, tr, th, or td.
        // TODO

        // Step 25. For each node in node list:
        // TODO

        // Step 26. If end node is an editable Text node, call deleteData(0, end offset) on it.
        // TODO

        // Step 27. Canonicalize whitespace at the active range's start, with fix collapsed space false.
        // TODO

        // Step 28. Canonicalize whitespace at the active range's end, with fix collapsed space false.
        // TODO

        // Step 29.
        //
        // This step does not exist in the spec

        // Step 30. If block merging is false, or start block or end block is null, or start block is not
        // in the same editing host as end block, or start block and end block are the same:
        // TODO

        // Step 31. If start block has one child, which is a collapsed block prop, remove its child from it.
        // TODO

        // Step 32. If start block is an ancestor of end block:
        // TODO

        // Step 33. Otherwise, if start block is a descendant of end block:
        // TODO

        // Step 34. Otherwise:
        // TODO

        // Step 35.
        //
        // This step does not exist in the spec

        // Step 36. Let ancestor be start block.
        // TODO

        // Step 37. While ancestor has an inclusive ancestor ol in the same editing host whose nextSibling is
        // also an ol in the same editing host, or an inclusive ancestor ul in the same editing host whose nextSibling
        // is also a ul in the same editing host:
        // TODO

        // Step 38. Restore the values from values.
        // TODO

        // Step 39. If start block has no children, call createElement("br") on the context object and
        // append the result as the last child of start block.
        // TODO

        // Step 40. Remove extraneous line breaks at the end of start block.
        // TODO

        // Step 41. Restore states and values from overrides.
        // TODO
    }
}
