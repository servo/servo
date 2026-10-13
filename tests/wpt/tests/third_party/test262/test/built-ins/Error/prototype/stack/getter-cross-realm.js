// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  The getter's [[ErrorData]] check is realm-agnostic: a getter from realm A
  invoked on an Error instance from realm B returns a String, since the
  internal-slot check examines object identity of the slot, not realm origin.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If E does not have an [[ErrorData]] internal slot, return undefined.
  4. Return an implementation-defined string that represents the stack trace of E.
features: [error-stack-accessor, cross-realm]
---*/

var getA = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

var realmB = $262.createRealm().global;

assert.notSameValue(
  Error.prototype,
  realmB.Error.prototype,
  'precondition: the two realms have distinct Error.prototype objects'
);

// (a) Realm A's getter on a realm B Error instance: the instance has
// [[ErrorData]], so the getter returns a string regardless of which realm
// owns the slot.
var errB = new realmB.Error('msg');
assert.sameValue(typeof getA.call(errB), 'string', 'cross-realm Error instance returns a string');

// (b) Realm A's getter on a realm B plain object: no [[ErrorData]], returns
// undefined.
var plainB = new realmB.Object();
assert.sameValue(getA.call(plainB), undefined, 'cross-realm plain object returns undefined');

// (c) Realm A's getter on realm B's Error.prototype: the prototype object
// itself has no [[ErrorData]] slot, so the getter returns undefined.
assert.sameValue(getA.call(realmB.Error.prototype), undefined, 'cross-realm Error.prototype returns undefined');

// (d) Realm B's getter on a realm A Error instance: same logic in reverse.
var getB = Object.getOwnPropertyDescriptor(realmB.Error.prototype, 'stack').get;
assert.sameValue(typeof getB, 'function', 'realm B has its own getter');
assert.notSameValue(getA, getB, 'the two realms have distinct getter functions');

var errA = new Error('msg');
assert.sameValue(typeof getB.call(errA), 'string', 'realm B getter on realm A Error instance returns a string');
