/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/
 *
 * To the extent possible under law, the editors have waived all copyright
 * and related or neighboring rights to this work.
 */

// https://dom.spec.whatwg.org/#text
[Exposed=Window]
interface Text : CharacterData {
  [Throws] constructor(optional DOMString data = "");
  [NewObject, Throws]
  Text splitText(unsigned long offset);
  [Pure]
  readonly attribute DOMString wholeText;
};
