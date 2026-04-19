// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class subclass binding
---*/
class Base {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }
}

var obj = {};
class Subclass extends Base {
  constructor(x, y) {
    super(x,y);
    assert.sameValue(this !== obj, true, "The result of `this !== obj` is `true`");
  }
}

var f = Subclass.bind(obj);
assert.throws(TypeError, function () { f(1, 2); });
var s = new f(1, 2);
assert.sameValue(s.x, 1, "The value of `s.x` is `1`");
assert.sameValue(s.y, 2, "The value of `s.y` is `2`");
assert.sameValue(
  Object.getPrototypeOf(s),
  Subclass.prototype,
  "`Object.getPrototypeOf(s)` returns `Subclass.prototype`"
);

var s1 = new f(1);
assert.sameValue(s1.x, 1, "The value of `s1.x` is `1`");
assert.sameValue(s1.y, undefined, "The value of `s1.y` is `undefined`");
assert.sameValue(
  Object.getPrototypeOf(s1),
  Subclass.prototype,
  "`Object.getPrototypeOf(s1)` returns `Subclass.prototype`"
);

var g = Subclass.bind(obj, 1);
assert.throws(TypeError, function () { g(8); });
var s2 = new g(8);
assert.sameValue(s2.x, 1, "The value of `s2.x` is `1`");
assert.sameValue(s2.y, 8, "The value of `s2.y` is `8`");
assert.sameValue(
  Object.getPrototypeOf(s),
  Subclass.prototype,
  "`Object.getPrototypeOf(s)` returns `Subclass.prototype`"
);
