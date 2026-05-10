// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change to non-configurable, non-enumerable and non-writable
info: |
    Change the descriptor using [[DefineOwnProperty]] to {configurable: false, enumerable: false},
    set arguments[0] = 2 and then change property descriptor to {writable: false}.
    The descriptor's enumerable property is the one set before the mapping removal.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function fn(a) {
  Object.defineProperty(arguments, "0", {configurable: false, enumerable: false});
  arguments[0] = 2;
  Object.defineProperty(arguments, "0", {writable: false});

  assert.sameValue(a, 2);

  verifyProperty(arguments, "0", {
    value: 2,
    writable: false,
    enumerable: false,
    configurable: false,
  });

  // Postcondition: Arguments mapping is removed.
  a = 3;

  verifyProperty(arguments, "0", {
    value: 2,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}
fn(1);

