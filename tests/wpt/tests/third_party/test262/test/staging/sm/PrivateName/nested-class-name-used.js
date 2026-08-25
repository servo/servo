// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// AllPrivateIdentifiersValid uses only lexical string names, not
// the dynamic private names; which means the below is not a syntax
// error but is instead a TypeError on access.

class A {
  #x = 10;
  f() {
    class B {
      g() {
        return this.#x;  // note: #x isn't declared in this class, but
                         // the enclosing.
      }
    };
    this.y = new B;
  }
  constructor() {
    this.f();
  }
  g() {
    return this.y.g();
  }
};

var a = new A;
assert.throws(TypeError, function() {
  a.g();
});
