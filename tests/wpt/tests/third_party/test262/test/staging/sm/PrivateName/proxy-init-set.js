// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Ensure that the distinction between Proxy Init and Proxy Set holds

var target = {};
var p1 = new Proxy(target, {});
var p2 = new Proxy(target, {});

class Base {
  constructor(o) {
    return o;
  }
}

class A extends Base {
  #field = 10;
  static gf(o) {
    return o.#field;
  }
  static sf(o) {
    o.#field = 15;
  }
}

class B extends Base {
  #field = 25;
  static gf(o) {
    return o.#field;
  }
  static sf(o) {
    o.#field = 20;
  }
}

// Verify field handling on the proxy we install it on.
new A(p1);
assert.sameValue(A.gf(p1), 10);
A.sf(p1)
assert.sameValue(A.gf(p1), 15);

// Despite P1 being stamped with A's field, it shouldn't
// be sufficient to set B's field.
assert.throws(TypeError, () => B.sf(p1));
assert.throws(TypeError, () => B.gf(p1));
assert.throws(TypeError, () => B.sf(p1));
new B(p1);
assert.sameValue(B.gf(p1), 25);
B.sf(p1);
assert.sameValue(B.gf(p1), 20);

// A's field should't be on the target
assert.throws(TypeError, () => A.gf(target));

// Can't set the field, doesn't exist
assert.throws(TypeError, () => A.sf(p2));

// Definitely can't get the field, doesn't exist.
assert.throws(TypeError, () => A.gf(p2));

// Still should't be on the target.
assert.throws(TypeError, () => A.gf(target));
