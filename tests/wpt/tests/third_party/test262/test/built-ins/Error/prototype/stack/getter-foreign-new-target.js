// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  An Error instance constructed with a non-Error new.target acquires the
  [[ErrorData]] internal slot from Error's [[Construct]], so the getter
  returns a String when invoked directly. Property access via the result
  does NOT find the inherited accessor, because the result's [[Prototype]]
  is the new.target's prototype, not Error.prototype.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
includes: [nativeErrors.js]
features: [error-stack-accessor, Reflect.construct]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

function NotAnError() {}

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var e = Reflect.construct(Ctor, ['msg'], NotAnError);

  assert.sameValue(
    Object.getPrototypeOf(e),
    NotAnError.prototype,
    Ctor.name + ': [[Prototype]] is the new.target prototype'
  );

  // The internal slot is set by Ctor's [[Construct]] regardless of new.target,
  // so the getter (called directly) still produces a string.
  assert.sameValue(typeof get.call(e), 'string', Ctor.name + ': via get.call');

  // Property access on the instance does NOT find the inherited accessor:
  // Error.prototype is not on e's prototype chain (it's only on Ctor.prototype,
  // which is bypassed by the foreign new.target). The lookup walks
  // NotAnError.prototype then Object.prototype and returns undefined.
  assert.sameValue(e.stack, undefined, Ctor.name + ': property access returns undefined');
}
