// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Mapped arguments object with non-configurable property
description: >
    Mapped arguments property is changed to non-configurable and
    non-writable. Perform property attribute changes with two
    [[DefineOwnProperty]] calls. Add intervening call to
    SetMutableBinding.
flags: [noStrict]
---*/

function argumentsNonConfigurableThenNonWritableWithInterveningSetMutableBinding(a) {
  Object.defineProperty(arguments, "0", {configurable: false});
  a = 2;
  Object.defineProperty(arguments, "0", {writable: false});
  assert.sameValue(a, 2);
  // `arguments[0] === 1` per ES2015, Rev 38, April 14, 2015 Final Draft.
  // Specification bug: https://bugs.ecmascript.org/show_bug.cgi?id=4371
  assert.sameValue(arguments[0], 2);

  // Postcondition: Arguments mapping is removed.
  a = 3;
  assert.sameValue(a, 3);
  assert.sameValue(arguments[0], 2);
}
argumentsNonConfigurableThenNonWritableWithInterveningSetMutableBinding(1);
