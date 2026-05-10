// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change with non-configurable, non-enumerable and non-writable property
info: |
    Mapping stop working when property is set to non-writable. The
    descriptor's enumerable property is the one set before the mapping removal.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function fn(a) {
  Object.defineProperty(arguments, "0", {configurable: false, enumerable: false, writable: false});

  // Postcondition: Arguments mapping is removed.
  a = 2;

  verifyProperty(arguments, "0", {
    value: 1,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}
fn(1);

