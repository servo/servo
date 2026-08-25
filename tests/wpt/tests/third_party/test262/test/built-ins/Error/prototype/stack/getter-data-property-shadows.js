// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  An own data property named "stack" on an Error instance shadows the inherited
  accessor when accessed via property lookup.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');
  Object.defineProperty(err, 'stack', {
    value: 'sentinel',
    writable: true,
    enumerable: true,
    configurable: true,
  });

  assert.sameValue(err.stack, 'sentinel', Ctor.name + ': own data property is returned by [[Get]]');

  // The inherited accessor still produces a string when invoked directly,
  // because the algorithm operates on the [[ErrorData]] slot, not on properties.
  assert.sameValue(typeof get.call(err), 'string', Ctor.name + ': inherited accessor still returns a string');
}
