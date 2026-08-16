// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  Returns undefined when called on an object that does not have an
  [[ErrorData]] internal slot.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
features: [error-stack-accessor]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

var receivers = [
  ['plain object', {}],
  ['null-prototype object', { __proto__: null }],
  ['array', []],
  ['function', function () {}],
  ['RegExp', /re/],
  ['Date', new Date()],
  ['Boolean wrapper', new Boolean(true)],
  ['Number wrapper', new Number(0)],
  ['String wrapper', new String('')],
  typeof ArrayBuffer === 'undefined' ? null : ['ArrayBuffer', new ArrayBuffer(0)],
  typeof Map === 'undefined' ? null : ['Map', new Map()],
  typeof Set === 'undefined' ? null : ['Set', new Set()],
  typeof WeakMap === 'undefined' ? null : ['WeakMap', new WeakMap()],
  typeof WeakSet === 'undefined' ? null : ['WeakSet', new WeakSet()],
  typeof Promise === 'undefined' ? null : ['Promise', new Promise(function () {})],
  typeof Int8Array === 'undefined' ? null : ['TypedArray', new Int8Array()]
];

for (var i = 0; i < receivers.length; ++i) {
  if (!receivers[i]) continue;
  var label = receivers[i][0];
  var value = receivers[i][1];
  assert.sameValue(get.call(value), undefined, label);
}

// An object that inherits from Error.prototype but lacks [[ErrorData]] still
// returns undefined: the getter checks for the internal slot, not the prototype.
var fakeError = Object.create(Error.prototype);
assert.sameValue(get.call(fakeError), undefined, 'object with Error.prototype on its prototype chain');

var fakeErrorWithStack = Object.create(Error.prototype);
Object.defineProperty(fakeErrorWithStack, 'stack', { value: 'imposter', writable: true, enumerable: true, configurable: true });
assert.sameValue(
  get.call(fakeErrorWithStack),
  undefined,
  'an existing own "stack" data property is ignored by the getter on a non-Error object'
);
