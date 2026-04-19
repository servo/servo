// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.2.2
description: >
    [[Construct]] ( argumentsList, newTarget)

    ...
    13. If result.[[type]] is return, then
      ...
      b. If kind is "base", return NormalCompletion(thisArgument).
      ...
    ...

    `return this;`

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

    return this;
  }
}

var object = new Derived();

// super is called
assert.sameValue(calls, 1, "The value of `calls` is `1`, because `super()`");

// The this object was returned.
assert.sameValue(object.prop, 1);
assert.sameValue(object instanceof Derived, true);
assert.sameValue(object instanceof Base, true);
