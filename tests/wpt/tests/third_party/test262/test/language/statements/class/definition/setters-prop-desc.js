// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions
es6id: 14.5
description: Class methods - "set" accessors
includes: [propertyHelper.js]
---*/

function assertSetterDescriptor(object, name) {
  var descr = Object.getOwnPropertyDescriptor(object, name);
  verifyProperty(object, name, {
    enumerable: false,
    configurable: true,
  });
  assert.sameValue(typeof descr.set, 'function', "`typeof descr.set` is `'function'`");
  assert.sameValue('prototype' in descr.set, false, "The result of `'prototype' in descr.set` is `false`");
  assert.sameValue(descr.get, undefined, "The value of `descr.get` is `undefined`");
}

var x, staticX, y, staticY;
class C {
  set x(v) { x = v; }
  static set staticX(v) { staticX = v; }
  set y(v) { y = v; }
  static set staticY(v) { staticY = v; }
}

assert.sameValue(new C().x = 1, 1, "`new C().x = 1` is `1`");
assert.sameValue(x, 1, "The value of `x` is `1`");
assert.sameValue(C.staticX = 2, 2, "`C.staticX = 2` is `2`");
assert.sameValue(staticX, 2, "The value of `staticX` is `2`");
assert.sameValue(new C().y = 3, 3, "`new C().y = 3` is `3`");
assert.sameValue(y, 3, "The value of `y` is `3`");
assert.sameValue(C.staticY = 4, 4, "`C.staticY = 4` is `4`");
assert.sameValue(staticY, 4, "The value of `staticY` is `4`");

assertSetterDescriptor(C.prototype, 'x');
assertSetterDescriptor(C.prototype, 'y');
assertSetterDescriptor(C, 'staticX');
assertSetterDescriptor(C, 'staticY');
