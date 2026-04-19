// Copyright (C) 2017 Claude Pache. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-invariants-of-the-essential-internal-methods
description: >
  A property made non-writable, non-configurable must not be reported as writable
  ("$1" property of the RegExp built-in)
info: |
  [[GetOwnProperty]] (P)
  [...]
  - If the [[Writable]] attribute may change from false to true,
    then the [[Configurable]] attribute must be true..
  [...]
  (This invariant was violated for the specific property under test by at least
  one implementation as of January 2017.)
---*/

if (Reflect.defineProperty(RegExp, '$1', {
    writable: false,
    configurable: false
  })) {
  var desc = Reflect.getOwnPropertyDescriptor(RegExp, '$1');
  assert.sameValue(desc.writable, false);
}
