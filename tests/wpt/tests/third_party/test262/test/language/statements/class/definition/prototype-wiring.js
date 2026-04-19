// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class prototype wiring
---*/

class Base {
  constructor(x) {
    this.foobar = x;
  }
}

class Subclass extends Base {
  constructor(x) {
    super(x);
  }
}

var s = new Subclass(1);
assert.sameValue(s.foobar, 1, "The value of `s.foobar` is `1`");
assert.sameValue(
  Object.getPrototypeOf(s),
  Subclass.prototype,
  "`Object.getPrototypeOf(s)` returns `Subclass.prototype`"
);

var s1 = new Subclass(1, 2);
assert.sameValue(s1.foobar, 1, "The value of `s1.foobar` is `1`");
assert.sameValue(
  Object.getPrototypeOf(s1) === Subclass.prototype,
  true,
  "The result of `Object.getPrototypeOf(s1) === Subclass.prototype` is `true`"
);

var s2 = new Subclass();
assert.sameValue(s2.foobar, undefined, "The value of `s2.foobar` is `undefined`");
assert.sameValue(
  Object.getPrototypeOf(s2),
  Subclass.prototype,
  "`Object.getPrototypeOf(s2)` returns `Subclass.prototype`"
);
assert.throws(TypeError, function() { Subclass(1); });
assert.throws(TypeError, function() { Subclass(1,2,3,4); });

class Subclass2 extends Subclass {
  constructor() {
    super(5, 6, 7);
  }
}

var ss2 = new Subclass2();
assert.sameValue(ss2.foobar, 5, "The value of `ss2.foobar` is `5`");
assert.sameValue(
  Object.getPrototypeOf(ss2),
  Subclass2.prototype,
  "`Object.getPrototypeOf(ss2)` returns `Subclass2.prototype`"
);

class Subclass3 extends Base {
  constructor(x, y) {
    super(x + y);
  }
}

var ss3 = new Subclass3(27,42-27);
assert.sameValue(ss3.foobar, 42, "The value of `ss3.foobar` is `42`");
assert.sameValue(
  Object.getPrototypeOf(ss3),
  Subclass3.prototype,
  "`Object.getPrototypeOf(ss3)` returns `Subclass3.prototype`"
);
