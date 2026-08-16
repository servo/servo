// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  The setter's home-object (SameValue) check is realm-scoped: a setter from
  realm A locks against realm A's %Error.prototype% only. Cross-realm objects
  are accepted as receivers (they pass the SameValue check), and an Error
  instance from another realm gets an own "stack" data property installed.
info: |
  set Error.prototype.stack

  [...]
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  2. If SameValue(this, home) is true, then
    a. Throw a TypeError exception.
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
includes: [propertyHelper.js]
features: [error-stack-accessor, cross-realm]
---*/

var setA = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

var realmB = $262.createRealm().global;

assert.notSameValue(
  Error.prototype,
  realmB.Error.prototype,
  'precondition: the two realms have distinct Error.prototype objects'
);

// (a) Realm A's setter on a realm B Error instance: passes SameValue (the
// instance is not realm A's Error.prototype), and the instance has no own
// "stack" property at construction, so CreateDataPropertyOrThrow installs one.
var errB = new realmB.Error('msg');
assert.sameValue(
  Object.prototype.hasOwnProperty.call(errB, 'stack'),
  false,
  'precondition: cross-realm Error instance has no own "stack" property'
);

setA.call(errB, 'sentinel');

verifyProperty(errB, 'stack', {
  value: 'sentinel',
  writable: true,
  enumerable: true,
  configurable: true,
});

// (b) Realm A's setter on a plain object from realm B: same path, installs an
// own data property.
var plainB = new realmB.Object();
setA.call(plainB, 'plain');

verifyProperty(plainB, 'stack', {
  value: 'plain',
  writable: true,
  enumerable: true,
  configurable: true,
});

// (c) Realm A's setter is structurally distinct from realm B's setter.
var setB = Object.getOwnPropertyDescriptor(realmB.Error.prototype, 'stack').set;
assert.sameValue(typeof setB, 'function', 'realm B has its own setter');
assert.notSameValue(setA, setB, 'the two realms have distinct setter functions');

// (d) Realm A's setter on realm B's Error.prototype passes realm A's
// SameValue check (the two prototypes are distinct objects), so step 2 of
// SetterThatIgnoresPrototypeProperties does not fire. Step 3 finds realm B's
// own "stack" accessor descriptor and proceeds to step 5: Set(O, p, v, true)
// invokes that accessor's setter, which is realm B's setter. Realm B's setter
// then throws via its own SameValue check (this === realm B's Error.prototype).
// The thrown TypeError is realm B's TypeError.
assert.throws(realmB.TypeError, function () {
  setA.call(realmB.Error.prototype, 'should-throw');
}, 'realm A setter on realm B Error.prototype throws (via realm B setter)');

// (e) Realm A's setter on realm A's own Error.prototype throws via step 2
// (SameValue with realm A's home).
assert.throws(TypeError, function () {
  setA.call(Error.prototype, 'should-throw');
}, 'realm A setter on realm A Error.prototype throws via SameValue');
