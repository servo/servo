// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws TypeError when attempting to overwrite a private method.
esid: sec-privateset
info: |
  7.3.30 PrivateSet ( P, O, value )
  1. Let entry be ! PrivateElementFind(P, O).
  2. If entry is empty, throw a TypeError exception.
  3. If entry.[[Kind]] is field, then
    ...
  4. Else if entry.[[Kind]] is method, then
    a. Throw a TypeError exception.
  5. ...

features: [class, class-methods-private]
---*/

class C {
  #m() {}

  assign() {
    this.#m = 0;
  }
}

var obj = new C();

assert.throws(TypeError, function() {
  obj.assign();
});
