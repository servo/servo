/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.is{Sealed,Frozen}, Object.{seal,freeze}
info: bugzilla.mozilla.org/show_bug.cgi?id=492849
esid: pending
---*/

// Empty object

var o1 = {};

assert.sameValue(Object.isExtensible(o1), true);
assert.sameValue(Object.isSealed(o1), false);
assert.sameValue(Object.isFrozen(o1), false);

Object.preventExtensions(o1);

// An non-extensible empty object has no properties, so it is vacuously sealed
// and frozen.
assert.sameValue(Object.isExtensible(o1), false);
assert.sameValue(Object.isSealed(o1), true);
assert.sameValue(Object.isFrozen(o1), true);


// Object with a data property

var o2 = { 1: 2 };

assert.sameValue(Object.isExtensible(o2), true);
assert.sameValue(Object.isSealed(o2), false);
assert.sameValue(Object.isFrozen(o2), false);

Object.preventExtensions(o2);

assert.sameValue(Object.isExtensible(o2), false);
assert.sameValue(Object.isSealed(o2), false);
assert.sameValue(Object.isFrozen(o2), false);

Object.seal(o2);

assert.sameValue(Object.isExtensible(o2), false);
assert.sameValue(Object.isSealed(o2), true);
assert.sameValue(Object.isFrozen(o2), false);

assert.sameValue(o2[1], 2);

var desc;

desc = Object.getOwnPropertyDescriptor(o2, "1");
assert.sameValue(typeof desc, "object");
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, 2);
assert.sameValue(desc.writable, true);

o2[1] = 17;

assert.sameValue(o2[1], 17);

desc = Object.getOwnPropertyDescriptor(o2, "1");
assert.sameValue(typeof desc, "object");
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, 17);
assert.sameValue(desc.writable, true);

Object.freeze(o2);

assert.sameValue(o2[1], 17);

desc = Object.getOwnPropertyDescriptor(o2, "1");
assert.sameValue(typeof desc, "object");
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, 17);
assert.sameValue(desc.writable, false);


// Object with an accessor property

var o3 = { get foo() { return 17; } };

assert.sameValue(Object.isExtensible(o3), true);
assert.sameValue(Object.isSealed(o3), false);
assert.sameValue(Object.isFrozen(o3), false);

Object.preventExtensions(o3);

assert.sameValue(Object.isExtensible(o3), false);
assert.sameValue(Object.isSealed(o3), false);
assert.sameValue(Object.isFrozen(o3), false);

Object.seal(o3);

// An accessor property in a sealed object is unchanged if that object is
// frozen, so a sealed object containing only accessor properties is also
// vacuously frozen.
assert.sameValue(Object.isExtensible(o3), false);
assert.sameValue(Object.isSealed(o3), true);
assert.sameValue(Object.isFrozen(o3), true);

Object.freeze(o3);

assert.sameValue(Object.isExtensible(o3), false);
assert.sameValue(Object.isSealed(o3), true);
assert.sameValue(Object.isFrozen(o3), true);
