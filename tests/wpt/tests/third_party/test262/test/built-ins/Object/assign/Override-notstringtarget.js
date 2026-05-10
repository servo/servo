// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Test override of Object.Assign(target,...sources),
  Every string from sources will be wrapped to objects, and override from the first letter(result[0]) all the time
es6id:  19.1.2.1
---*/

var target = 12;
var result = Object.assign(target, "aaa", "bb2b", "1c");

assert.sameValue(Object.getOwnPropertyNames(result).length, 4, "The length should be 4 in the final object.");
assert.sameValue(result[0], "1", "The value should be {\"0\":\"1\"}.");
assert.sameValue(result[1], "c", "The value should be {\"1\":\"c\"}.");
assert.sameValue(result[2], "2", "The value should be {\"2\":\"2\"}.");
assert.sameValue(result[3], "b", "The value should be {\"3\":\"b\"}.");
