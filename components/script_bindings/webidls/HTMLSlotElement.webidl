/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-slot-element
[Exposed=Window]
interface HTMLSlotElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions] attribute DOMString name;
  sequence<Node> assignedNodes(optional AssignedNodesOptions options = {});
  sequence<Element> assignedElements(optional AssignedNodesOptions options = {});
  undefined assign((Element or Text)... nodes);
};

dictionary AssignedNodesOptions {
  boolean flatten = false;
};

// https://dom.spec.whatwg.org/#mixin-slotable
interface mixin Slottable {
  readonly attribute HTMLSlotElement? assignedSlot;
};
Element includes Slottable;
Text includes Slottable;
