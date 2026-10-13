// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  The getter returns a String when called on a subclass of Error or any
  NativeError, since subclasses inherit the [[ErrorData]] internal slot.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
includes: [nativeErrors.js]
features: [error-stack-accessor, class]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var Sub = class extends Ctor {};
  var e = new Sub('msg');

  assert.sameValue(typeof get.call(e), 'string', Ctor.name + ': subclass via get.call');
  assert.sameValue(typeof e.stack, 'string', Ctor.name + ': subclass via property access');
}
