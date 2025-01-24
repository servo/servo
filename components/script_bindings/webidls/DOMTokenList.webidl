/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://dom.spec.whatwg.org/#domtokenlist
[Exposed=Window]
interface DOMTokenList {
  [Pure]
  readonly attribute unsigned long length;
  [Pure]
  getter DOMString? item(unsigned long index);

  [Pure]
  boolean contains(DOMString token);
  [CEReactions, Throws]
  undefined add(DOMString... tokens);
  [CEReactions, Throws]
  undefined remove(DOMString... tokens);
  [CEReactions, Throws]
  boolean toggle(DOMString token, optional boolean force);
  [CEReactions, Throws]
  boolean replace(DOMString token, DOMString newToken);
  [Pure, Throws]
  boolean supports(DOMString token);

  [CEReactions, Pure]
  stringifier attribute DOMString value;

  iterable<DOMString?>;
};
