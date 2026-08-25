// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws TypeError when attempting to install private methods multiple times.
esid: sec-privatemethodoraccessoradd
info: |
  7.3.28 PrivateMethodOrAccessorAdd ( method, O )
    1. Assert: method.[[Kind]] is either method or accessor.
    2. Let entry be ! PrivateElementFind(method.[[Key]], O).
    3. If entry is not empty, throw a TypeError exception.
    ...

features: [class, class-methods-private]
---*/

class Base {
  constructor(o) {
    return o;
  }
}

class C extends Base {
  #m() {}
}

var obj = {};

new C(obj);

assert.throws(TypeError, function() {
  new C(obj);
});
