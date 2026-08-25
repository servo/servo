// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.2.2
description: >
    [[Construct]] ( argumentsList, newTarget)

    ...
    13. If result.[[type]] is return, then
      a. If Type(result.[[value]]) is Object, return NormalCompletion(result.[[value]]).
      ...
    ...

    `return {};`

---*/
var calls = 0;
class Base {
  constructor() {
    this.prop = 1;
    calls++;
  }
}
class Derived extends Base {
  constructor() {
    super();

    return {};
  }
}

var object = new Derived();

// super is called
assert.sameValue(calls, 1, "The value of `calls` is `1`, because `super()`");

// But the this object was discarded.
assert.sameValue(typeof object.prop, "undefined");
assert.sameValue(object instanceof Derived, false);
assert.sameValue(object instanceof Base, false);
