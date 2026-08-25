// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change to non-writable, non-enumerable and non-configurable
info: |
    Change the descriptor using [[DefineOwnProperty]] to {writable: false, enumerable: false},
    set argument[0] = 2 and then change property descriptor to {configurable: false}.
    The descriptor's enumerable property continues with its configured value.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function fn(a) {
  Object.defineProperty(arguments, "0", {writable: false, enumerable: false});
  arguments[0] = 2;
  Object.defineProperty(arguments, "0", {configurable: false});

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: false,
  });

  // Postcondition: Arguments mapping is removed.
  a = 3;

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}
fn(1);
