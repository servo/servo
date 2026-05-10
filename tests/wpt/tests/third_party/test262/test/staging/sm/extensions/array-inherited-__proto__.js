/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  The [[Prototype]] of an object whose prototype chain contains an array isn't that array's [[Prototype]]
info: bugzilla.mozilla.org/show_bug.cgi?id=769041
esid: pending
---*/

var arr = [];
assert.sameValue(Array.isArray(arr), true);
var objWithArrPrototype = Object.create(arr);
assert.sameValue(!Array.isArray(objWithArrPrototype), true);
assert.sameValue(objWithArrPrototype.__proto__, arr);
var objWithArrGrandPrototype = Object.create(objWithArrPrototype);
assert.sameValue(!Array.isArray(objWithArrGrandPrototype), true);
assert.sameValue(objWithArrGrandPrototype.__proto__, objWithArrPrototype);
