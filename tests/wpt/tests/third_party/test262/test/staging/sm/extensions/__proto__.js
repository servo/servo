/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  __proto__ as accessor
info: bugzilla.mozilla.org/show_bug.cgi?id=770344
esid: pending
---*/

var protoDesc = Object.getOwnPropertyDescriptor(Object.prototype, "__proto__");
assert.sameValue(protoDesc !== null, true);
assert.sameValue(typeof protoDesc, "object");
assert.sameValue(protoDesc.hasOwnProperty("get"), true);
assert.sameValue(protoDesc.hasOwnProperty("set"), true);
assert.sameValue(protoDesc.hasOwnProperty("enumerable"), true);
assert.sameValue(protoDesc.hasOwnProperty("configurable"), true);
assert.sameValue(protoDesc.hasOwnProperty("value"), false);
assert.sameValue(protoDesc.hasOwnProperty("writable"), false);

assert.sameValue(protoDesc.configurable, true);
assert.sameValue(protoDesc.enumerable, false);
assert.sameValue(typeof protoDesc.get, "function", protoDesc.get + "");
assert.sameValue(typeof protoDesc.set, "function", protoDesc.set + "");

assert.sameValue(delete Object.prototype.__proto__, true);
assert.sameValue(Object.getOwnPropertyDescriptor(Object.prototype, "__proto__"),
         undefined);

var obj = {};
obj.__proto__ = 5;
assert.sameValue(Object.getPrototypeOf(obj), Object.prototype);
assert.sameValue(obj.hasOwnProperty("__proto__"), true);

var desc = Object.getOwnPropertyDescriptor(obj, "__proto__");
assert.sameValue(desc !== null, true);
assert.sameValue(typeof desc, "object");
assert.sameValue(desc.value, 5);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, true);
