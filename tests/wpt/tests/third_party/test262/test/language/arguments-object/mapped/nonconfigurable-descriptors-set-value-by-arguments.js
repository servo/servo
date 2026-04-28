// Copyright (C) 2017 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Mapped arguments property descriptor change with non-configurable property
info: |
    Mapping keep working when property is set to non-configurable and its
    value is changed using arguments[i] where "i" is the argument index.
flags: [noStrict]
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
includes: [propertyHelper.js]
---*/

function argumentsAndSetByIndex(a) {
  Object.defineProperty(arguments, "0", {configurable: false});

  arguments[0] = 2;

  assert.sameValue(a, 2);

  verifyProperty(arguments, "0", {
    value: 2,
    writable: true,
    enumerable: true,
    configurable: false,
  });
}
argumentsAndSetByIndex(1);

