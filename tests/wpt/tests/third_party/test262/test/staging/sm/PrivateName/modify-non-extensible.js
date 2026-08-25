// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - class
  - class-fields-private
  - class-fields-public
  - nonextensible-applies-to-private
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

// Returns the argument in the constructor to allow stamping private fields into
// arbitrary objects.
class OverrideBase {
  constructor(o) {
    return o;
  }
};

class A extends OverrideBase {
  #a = 1;
  g() {
    return this.#a
  }

  static gs(o) {
    return o.#a;
  }
  static inca(obj) {
    obj.#a++;
  }
}

var obj = {};
new A(obj);  // Add #a to obj, but not g.
Object.seal(obj);
assert.sameValue('g' in obj, false);
assert.sameValue(A.gs(obj), 1);
A.inca(obj);
assert.sameValue(A.gs(obj), 2);

// Ensure that the object remains non-extensible
obj.h = 'hi'
assert.sameValue('h' in obj, false);


Object.freeze(obj);
A.inca(obj);  // Despite being frozen, private names are modifiable.
assert.sameValue(A.gs(obj), 3);
assert.sameValue(Object.isFrozen(obj), true);

var proxy = new Proxy({}, {});
assert.sameValue(Object.isFrozen(proxy), false);

new A(proxy);
assert.sameValue(A.gs(proxy), 1);

// Note: this doesn't exercise the non-native object
// path in TestIntegrityLevel like you might expect.
//
// For that see below.
Object.freeze(proxy);
assert.sameValue(Object.isFrozen(proxy), true);

A.inca(proxy);
assert.sameValue(A.gs(proxy), 2)

var target = { a: 10 };
Object.freeze(target);
assert.throws(TypeError, function () {
  new A(target);
});
assert.sameValue(Object.isFrozen(target), true);

var getOwnKeys = [];
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: function (target, key) {
    getOwnKeys.push(key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
});

Object.isFrozen(proxy);
assert.sameValue(getOwnKeys.length, 1);
