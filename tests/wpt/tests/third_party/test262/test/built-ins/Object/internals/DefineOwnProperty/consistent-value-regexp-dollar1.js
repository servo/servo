// Copyright (C) 2017 Claude Pache. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-invariants-of-the-essential-internal-methods
description: >
  Value of non-writable, non-configurable data property must not change
  ("$1" property of the RegExp built-in)
info: |
  [[GetOwnProperty]] (P)
  [...]
  - If a property P is described as a data property with Desc.[[Value]] equal
    to v and Desc.[[Writable]] and Desc.[[Configurable]] are both false, then
    the SameValue must be returned for the Desc.[[Value]] attribute of the
    property on all future calls to [[GetOwnProperty]] ( P ).
  [...]
  (This invariant was violated for the specific property under test by at least
  one implementation as of January 2017.)
---*/

Reflect.defineProperty(RegExp, '$1', {
  writable: false,
  configurable: false
});

var desc = Reflect.getOwnPropertyDescriptor(RegExp, '$1');
if (desc && desc.configurable === false && desc.writable === false) {
  /(x)/.exec('x');
  var desc2 = Reflect.getOwnPropertyDescriptor(RegExp, '$1');
  assert.sameValue(desc.value, desc2.value);
}
