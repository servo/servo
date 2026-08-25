// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  Throws a TypeError if the receiver is %Error.prototype% itself.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  1. If this is not an Object, throw a TypeError exception.
  2. If SameValue(this, home) is true, then
    a. NOTE: Throwing here emulates assignment to a non-writable data property
       on the home object in strict mode code.
    b. Throw a TypeError exception.
  [...]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

assert.throws(TypeError, function () {
  set.call(Error.prototype, '');
}, 'set.call(Error.prototype, "")');

// Property assignment also throws. The setter throws unconditionally via
// SetterThatIgnoresPrototypeProperties; the throw originates inside the
// accessor function and propagates regardless of the caller's strict-mode
// flag. (test262 runs this file in both strict and sloppy modes by default.)
assert.throws(TypeError, function () {
  Error.prototype.stack = '';
}, 'assignment to Error.prototype.stack');

// The accessor descriptor on Error.prototype is unchanged.
var desc = Object.getOwnPropertyDescriptor(Error.prototype, 'stack');
assert.notSameValue(desc, undefined, 'Error.prototype still has its own "stack" property');
assert.sameValue(typeof desc.get, 'function', 'getter is still installed');
assert.sameValue(typeof desc.set, 'function', 'setter is still installed');
assert.sameValue(desc.value, undefined, 'descriptor has no value (still an accessor)');
assert.sameValue(desc.writable, undefined, 'descriptor has no writable (still an accessor)');
