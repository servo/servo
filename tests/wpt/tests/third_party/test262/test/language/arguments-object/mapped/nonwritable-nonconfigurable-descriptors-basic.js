// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change to non-writable and non-configurable
info: |
    Mapping stop working when property is set to non-writable. Here we change the
    descriptor using [[DefineOwnProperty]] to {writable: false} and then
    change property descriptor to {configurable: false} in sequence.
    The property descriptor need to be the one set before the property be
    configured as {writable: false}.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function fn(a) {
  Object.defineProperty(arguments, "0", {writable: false});
  Object.defineProperty(arguments, "0", {configurable: false});

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false,
  });

  // Postcondition: Arguments mapping is removed. Descriptors need to be the same
  // as above.
  a = 2;

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: true,
    configurable: false,
  });
}
fn(1);

