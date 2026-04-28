// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
description: >
  Index gets unmapped when redefined with accessor. Unmapped index is created.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  6. Let allowed be ? OrdinaryDefineOwnProperty(args, P, newArgDesc).
  7. If allowed is false, return false.
  8. If isMapped is true, then
    a. If IsAccessorDescriptor(Desc) is true, then
      i. Call map.[[Delete]](P).
    [...]
  9. Return true.
flags: [noStrict]
---*/

(function(a) {
  let setCalls = 0;
  Object.defineProperty(arguments, "0", {
    set(_v) { setCalls += 1; },
    enumerable: true,
    configurable: true,
  });

  arguments[0] = "foo";

  assert.sameValue(setCalls, 1);
  assert.sameValue(a, 0);
  assert.sameValue(arguments[0], undefined);


  Object.defineProperty(arguments, "1", {
    get: () => "bar",
    enumerable: true,
    configurable: true,
  });

  assert.sameValue(arguments[1], "bar");
})(0);
