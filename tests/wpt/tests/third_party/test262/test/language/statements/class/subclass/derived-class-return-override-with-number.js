// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.2.2
description: >
    [[Construct]] ( argumentsList, newTarget)

    ...
    13. If result.[[type]] is return, then
      ...
      c. If result.[[value]] is not undefined, throw a TypeError exception.
    ...

    `return 0;`

---*/
class Base {
  constructor() {}
}
class Derived extends Base {
  constructor() {
    super();

    return 0;
  }
}

assert.throws(TypeError, function() {
  new Derived();
});
