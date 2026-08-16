// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  Property assignment (e.g. err.stack = v) goes through the inherited setter.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.
includes: [propertyHelper.js, nativeErrors.js]
features: [error-stack-accessor]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

// Plain object inheriting the accessor: assignment installs an own data property.
var plain = Object.create(Error.prototype);
plain.stack = 'sentinel';

verifyProperty(plain, 'stack', {
  value: 'sentinel',
  writable: true,
  enumerable: true,
  configurable: true,
});

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');

  // Assigning a non-string still throws TypeError (step 3 of the setter).
  // The `err.stack = null` form covered the "release memory" pattern in V8/JSC
  // before this proposal; it now throws.
  assert.throws(TypeError, function () {
    err.stack = null;
  }, Ctor.name + ': null assignment');

  assert.throws(TypeError, function () {
    err.stack = 0;
  }, Ctor.name + ': numeric assignment');

  assert.throws(TypeError, function () {
    err.stack = undefined;
  }, Ctor.name + ': undefined assignment');

  // The failed setter does not affect the [[ErrorData]] slot: the inherited
  // accessor still produces a string (Issue 13: a failed setter must not
  // clobber the slot).
  assert.sameValue(typeof get.call(err), 'string', Ctor.name + ': [[ErrorData]] preserved across failed setters');

  // Assigning a string installs/updates the own data property.
  err.stack = 'updated';
  assert.sameValue(err.stack, 'updated', Ctor.name + ': string assignment is observable');
}

// Assigning to %Error.prototype%.stack always throws, because the setter
// function itself throws via SetterThatIgnoresPrototypeProperties; the throw
// originates inside the accessor function and propagates regardless of the
// caller's strict-mode flag. (test262 runs this file in both strict and
// sloppy modes by default, so the bare assignment exercises both.)
assert.throws(TypeError, function () {
  Error.prototype.stack = 'top-level';
});
