// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.14
description: >
  Throws a TypeError if proto is a Symbol
info: |
  26.1.14 Reflect.setPrototypeOf ( target, proto )

  ...
  2. If Type(proto) is not Object and proto is not null, throw a TypeError
  exception
  ...
features: [Reflect, Reflect.setPrototypeOf, Symbol]
---*/

var s = Symbol(1);
assert.throws(TypeError, function() {
  Reflect.setPrototypeOf({}, s);
});
