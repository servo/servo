/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#interface-abstractrange
 */

[Exposed=Window]
interface AbstractRange {
  [Pure]
  readonly attribute Node startContainer;
  [Pure]
  readonly attribute unsigned long startOffset;
  [Pure]
  readonly attribute Node endContainer;
  [Pure]
  readonly attribute unsigned long endOffset;
  [Pure]
  readonly attribute boolean collapsed;
};
