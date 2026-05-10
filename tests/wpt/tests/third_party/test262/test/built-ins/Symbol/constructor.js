// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.3.1
description: >
    Symbol constructor
features: [Symbol]
---*/
assert.sameValue(
  Object.getPrototypeOf(Symbol('66')).constructor,
  Symbol,
  "The value of `Object.getPrototypeOf(Symbol('66')).constructor` is `Symbol`"
);
assert.sameValue(
  Object.getPrototypeOf(Object(Symbol('66'))).constructor,
  Symbol,
  "The value of `Object.getPrototypeOf(Object(Symbol('66'))).constructor` is `Symbol`"
);
