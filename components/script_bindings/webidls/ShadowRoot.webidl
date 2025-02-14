/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://dom.spec.whatwg.org/#interface-shadowroot
 */

[Exposed=Window]
interface ShadowRoot : DocumentFragment {
  readonly attribute ShadowRootMode mode;
  // readonly attribute boolean delegatesFocus;
  readonly attribute SlotAssignmentMode slotAssignment;
  readonly attribute boolean clonable;
  // readonly attribute boolean serializable;
  readonly attribute Element host;
  attribute EventHandler onslotchange;
};


enum ShadowRootMode { "open", "closed"};
enum SlotAssignmentMode { "manual", "named" };

ShadowRoot includes DocumentOrShadowRoot;

// https://html.spec.whatwg.org/multipage/#dom-parsing-and-serialization
partial interface ShadowRoot {
  // [CEReactions] undefined setHTMLUnsafe((TrustedHTML or DOMString) html);
  // DOMString getHTML(optional GetHTMLOptions options = {});

  // [CEReactions] attribute (TrustedHTML or [LegacyNullToEmptyString] DOMString) innerHTML;
  [CEReactions] attribute [LegacyNullToEmptyString] DOMString innerHTML;
};
