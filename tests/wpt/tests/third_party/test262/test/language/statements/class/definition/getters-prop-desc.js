// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions
es6id: 14.5
description: Class methods - "get" accessors
includes: [propertyHelper.js]
---*/

function assertGetterDescriptor(object, name) {
  var desc = Object.getOwnPropertyDescriptor(object, name);
  verifyProperty(object, name, {
    enumerable: false,
    configurable: true,
  });
  assert.sameValue(typeof desc.get, 'function', "`typeof desc.get` is `'function'`");
  assert.sameValue('prototype' in desc.get, false, "The result of `'prototype' in desc.get` is `false`");
  assert.sameValue(desc.set, undefined, "The value of `desc.set` is `undefined`");
}

class C {
  get x() { return 1; }
  static get staticX() { return 2; }
  get y() { return 3; }
  static get staticY() { return 4; }
}

assert.sameValue(new C().x, 1, "The value of `new C().x` is `1`. Defined as `get x() { return 1; }`");
assert.sameValue(C.staticX, 2, "The value of `C.staticX` is `2`. Defined as `static get staticX() { return 2; }`");
assert.sameValue(new C().y, 3, "The value of `new C().y` is `3`. Defined as `get y() { return 3; }`");
assert.sameValue(C.staticY, 4, "The value of `C.staticY` is `4`. Defined as `static get staticY() { return 4; }`");

assertGetterDescriptor(C.prototype, 'x');
assertGetterDescriptor(C.prototype, 'y');
assertGetterDescriptor(C, 'staticX');
assertGetterDescriptor(C, 'staticY');
