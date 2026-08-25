// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  ReferenceError when returning from a derived class constructor without calling
  `super()` is thrown after the function body has been left, so an iterator
  return handler can still call `super()`.
---*/

var iter = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {done: false};
  },
  return() {
    // Calls |super()|.
    this.f();

    return {done: true};
  },
};

class C extends class {} {
  constructor() {
    iter.f = () => super();

    for (var k of iter) {
      return;
    }
  }
}

var o = new C();
assert.sameValue(typeof o, "object");
