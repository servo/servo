// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
description: >
  Index stays mapped when redefined with complete descriptor, which differs only
  by the [[Value]] field. Unmapped index is created.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  6. Let allowed be ? OrdinaryDefineOwnProperty(args, P, newArgDesc).
  7. If allowed is false, return false.
  8. If isMapped is true, then
    [...]
    b. Else,
      i. If Desc.[[Value]] is present, then
        1. Let setStatus be Set(map, P, Desc.[[Value]], false).
        2. Assert: setStatus is true because formal parameters mapped by argument objects are always writable.
  9. Return true.
flags: [noStrict]
---*/

(function(a) {
  Object.defineProperty(arguments, "0", {
    value: "foo",
    writable: true,
    enumerable: true,
    configurable: true,
  });

  assert.sameValue(a, "foo");
  assert.sameValue(arguments[0], "foo");


  Object.defineProperty(arguments, "1", {
    value: "bar",
    writable: true,
    enumerable: true,
    configurable: true,
  });

  assert.sameValue(arguments[1], "bar");
})(0);
