// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  The setter does not check for the [[ErrorData]] internal slot, so it works
  on any extensible object that is not %Error.prototype%.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.
includes: [propertyHelper.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

// Plain object: installs an own data property.
var plain = {};
set.call(plain, 'plain');
verifyProperty(plain, 'stack', {
  value: 'plain',
  writable: true,
  enumerable: true,
  configurable: true,
});

// Array: installs an own data property.
var arr = [];
set.call(arr, 'array');
verifyProperty(arr, 'stack', {
  value: 'array',
  writable: true,
  enumerable: true,
  configurable: true,
});

// Function: installs an own data property.
var fn = function () {};
set.call(fn, 'function');
verifyProperty(fn, 'stack', {
  value: 'function',
  writable: true,
  enumerable: true,
  configurable: true,
});

// Object whose prototype is Error.prototype but lacks [[ErrorData]]: still works.
var fakeError = Object.create(Error.prototype);
set.call(fakeError, 'fake');
verifyProperty(fakeError, 'stack', {
  value: 'fake',
  writable: true,
  enumerable: true,
  configurable: true,
});
