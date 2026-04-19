// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Test Object.Assign(target,...sources).
esid: sec-object.assign
---*/

//"a" will be an property of the final object and the value should be 1
var target = {
  a: 1
};
/*
"1a2c3" have own enumerable properties, so it Should be wrapped to objects;
{b:6} is an object,should be assigned to final object.
undefined and null should be ignored;
125 is a number,it cannot has own enumerable properties;
{a:"c"},{a:5} will override property a, the value should be 5.
*/
var result = Object.assign(target, "1a2c3", {
  a: "c"
}, undefined, {
  b: 6
}, null, 125, {
  a: 5
});

assert.sameValue(Object.getOwnPropertyNames(result).length, 7, "The length should be 7 in the final object.");
assert.sameValue(result.a, 5, "The value should be {a:5}.");
assert.sameValue(result[0], "1", "The value should be {\"0\":\"1\"}.");
assert.sameValue(result[1], "a", "The value should be {\"1\":\"a\"}.");
assert.sameValue(result[2], "2", "The value should be {\"2\":\"2\"}.");
assert.sameValue(result[3], "c", "The value should be {\"3\":\"c\"}.");
assert.sameValue(result[4], "3", "The value should be {\"4\":\"3\"}.");
assert.sameValue(result.b, 6, "The value should be {b:6}.");
