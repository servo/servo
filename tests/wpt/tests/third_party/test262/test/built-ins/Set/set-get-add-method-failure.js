// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    Set ( [ iterable ] )

    When the Set function is called with optional argument iterable the following steps are taken:

    ...
    6. If iterable is either undefined or null, let iter be undefined.
    7. Else,
      a. Let adder be Get(set, "add").
      b. ReturnIfAbrupt(adder).
---*/

function MyError() {}
Object.defineProperty(Set.prototype, 'add', {
  get: function() {
    throw new MyError();
  }
});

new Set();

assert.throws(MyError, function() {
  new Set([]);
});
