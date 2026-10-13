// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  An object whose prototype is an Error instance does not itself have an
  [[ErrorData]] internal slot, so the getter returns undefined. The slot is
  not inherited through the prototype chain.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
includes: [nativeErrors.js]
features: [error-stack-accessor, __proto__]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var protoErr = new Ctor('outer');
  var o = { __proto__: protoErr };

  assert.sameValue(get.call(o), undefined, Ctor.name + ': get.call on object with instance as proto');

  // Property access walks the prototype chain to find the accessor on
  // Error.prototype, then calls the getter with this set to the original
  // receiver (o). Because o lacks [[ErrorData]], the result is undefined.
  assert.sameValue(o.stack, undefined, Ctor.name + ': property access on object with instance as proto');

  // The inherited instance still produces a string when the getter is invoked
  // on it directly.
  assert.sameValue(typeof get.call(protoErr), 'string', Ctor.name + ': get.call on the underlying instance');
}
