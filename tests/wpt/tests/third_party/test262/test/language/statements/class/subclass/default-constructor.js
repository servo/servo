// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class default constructor
---*/
var calls = 0;
class Base {
  constructor() {
    calls++;
  }
}
class Derived extends Base {}
var object = new Derived();
assert.sameValue(calls, 1, "The value of `calls` is `1`");

calls = 0;
assert.throws(TypeError, function() { Derived(); });
assert.sameValue(calls, 0, "The value of `calls` is `0`");
