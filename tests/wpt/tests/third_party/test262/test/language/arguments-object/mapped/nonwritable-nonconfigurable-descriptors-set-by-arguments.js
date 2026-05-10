// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change to non-writable and non-configurable
info: |
    Mapping stop working when property is set to non-writable. Change the
    descriptor using [[DefineOwnProperty]] to {writable: false}, set argument[0] = 2 and then
    change property descriptor to {configurable: false}.
    The descriptor's value is the one set before the property be
    configured as {writable: false} because mapping was removed.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function fn(a) {
  Object.defineProperty(arguments, "0", {writable: false});
  arguments[0] = 2;
  Object.defineProperty(arguments, "0", {configurable: false});

  assert.sameValue(a, 1);

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false,
  });

  // Postcondition: Arguments mapping is removed.
  a = 3;

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false,
  });
}
fn(1);

