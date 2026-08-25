// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class default constructor 2
---*/
class Base1 { }
assert.throws(TypeError, function() { Base1(); });

class Subclass1 extends Base1 { }

assert.throws(TypeError, function() { Subclass1(); });

var s1 = new Subclass1();
assert.sameValue(
  Subclass1.prototype,
  Object.getPrototypeOf(s1),
  "The value of `Subclass1.prototype` is `Object.getPrototypeOf(s1)`, after executing `var s1 = new Subclass1();`"
);

class Base2 {
  constructor(x, y) {
    this.x = x;
    this.y = y;
  }
}

class Subclass2 extends Base2 {};

var s2 = new Subclass2(1, 2);

assert.sameValue(
  Subclass2.prototype,
  Object.getPrototypeOf(s2),
  "The value of `Subclass2.prototype` is `Object.getPrototypeOf(s2)`, after executing `var s2 = new Subclass2(1, 2);`"
);
assert.sameValue(s2.x, 1, "The value of `s2.x` is `1`");
assert.sameValue(s2.y, 2, "The value of `s2.y` is `2`");

var f = Subclass2.bind({}, 3, 4);
var s2prime = new f();
assert.sameValue(
  Subclass2.prototype,
  Object.getPrototypeOf(s2prime),
  "The value of `Subclass2.prototype` is `Object.getPrototypeOf(s2prime)`"
);
assert.sameValue(s2prime.x, 3, "The value of `s2prime.x` is `3`");
assert.sameValue(s2prime.y, 4, "The value of `s2prime.y` is `4`");


var obj = {};
class Base3 {
  constructor() {
    return obj;
  }
}

class Subclass3 extends Base3 {};

var s3 = new Subclass3();
assert.sameValue(s3, obj, "The value of `s3` is `obj`");

