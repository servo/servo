// Copyright (C) 2017 Claude Pache. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-invariants-of-the-essential-internal-methods
description: >
  Value of non-writable, non-configurable data property must not change
  ("caller" property of a non-strict function)
info: |
  [[GetOwnProperty]] (P)
  [...]
  - If a property P is described as a data property with Desc.[[Value]] equal
    to v and Desc.[[Writable]] and Desc.[[Configurable]] are both false, then
    the SameValue must be returned for the Desc.[[Value]] attribute of the
    property on all future calls to [[GetOwnProperty]] ( P ).
  [...]
  (This invariant was violated for the specific property under test by a number
  of implementations as of January 2017.)
---*/

function f() {
  return Reflect.getOwnPropertyDescriptor(f, 'caller');
}

function g() {
  return f();
}

Reflect.defineProperty(f, 'caller', {
  writable: false,
  configurable: false
});

var desc = Reflect.getOwnPropertyDescriptor(f, 'caller');
if (desc && desc.configurable === false && desc.writable === false) {
  var desc2 = g();
  assert.sameValue(desc.value, desc2.value);
}
