// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    with new target
includes: [compareArray.js]
---*/
class Base {
  constructor(...a) {
    assert.sameValue(
      arguments.length,
      a.length,
      "The value of `arguments.length` is `a.length`"
    );
    this.base = a;
    var args = [];
    for (var i = 0; i < arguments.length; ++i) {
      args.push(arguments[i]);
    }
    assert.compareArray(args, a);
  }
}
class Child extends Base {
  constructor(...b) {
    super(1, 2, 3);
    assert.sameValue(
      arguments.length,
      b.length,
      "The value of `arguments.length` is `b.length`"
    );
    this.child = b;
    var args = [];
    for (var i = 0; i < arguments.length; ++i) {
      args.push(arguments[i]);
    }
    assert.compareArray(args, b);
  }
}

var c = new Child(1, 2, 3);

assert.sameValue(c.child.length, 3, "The value of `c.child.length` is `3`");
assert.sameValue(c.base.length, 3, "The value of `c.base.length` is `3`");

assert.compareArray(
  c.child,
  [1, 2, 3]
);
assert.compareArray(
  c.base,
  [1, 2, 3]
);
