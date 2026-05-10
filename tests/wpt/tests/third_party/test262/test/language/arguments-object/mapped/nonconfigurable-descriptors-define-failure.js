// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arguments-exotic-objects-defineownproperty-p-desc
description: >
  OrdinaryDefineOwnProperty returning `false` doesn't leave `arguments` in a
  corrupted state, for both mapped and unmapped indices.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  6. Let allowed be ? OrdinaryDefineOwnProperty(args, P, newArgDesc).
  7. If allowed is false, return false.
flags: [noStrict]
---*/

(function(a) {
  Object.defineProperty(arguments, "0", {configurable: false});

  assert.throws(TypeError, () => {
    Object.defineProperty(arguments, "0", {configurable: true});
  });

  a = 2;
  assert.sameValue(arguments[0], 2);


  Object.defineProperty(arguments, "1", {
    get: () => 3,
    configurable: false,
  });

  assert.throws(TypeError, () => {
    Object.defineProperty(arguments, "1", {value: "foo"});
  });

  assert.sameValue(arguments[1], 3);
  assert.throws(TypeError, () => {
    "use strict";
    delete arguments[1];
  });
})(0);
