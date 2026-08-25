// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    for-in to acquire properties from array
---*/
function props(x) {
  var array = [];
  for (let p in x) array.push(p);
  return array;
}
var subject;

subject = props([]);
assert.sameValue(subject.length, 0, "[]: length");
assert.sameValue(subject[0], undefined, "[]: first property");
assert.sameValue(subject[1], undefined, "[]: second property");
assert.sameValue(subject[2], undefined, "[]: third property");
assert.sameValue(subject[3], undefined, "[]: fourth property");

subject = props([1]);
assert.sameValue(subject.length, 1, "[1]: length");
assert.sameValue(subject[0], "0", "[1]: first property");
assert.sameValue(subject[1], undefined, "[1]: second property");
assert.sameValue(subject[2], undefined, "[1]: third property");
assert.sameValue(subject[3], undefined, "[1]: fourth property");

subject = props([1, 2]);
assert.sameValue(subject.length, 2, "[1, 2]: length");
assert.sameValue(subject[0], "0", "[1, 2]: first property");
assert.sameValue(subject[1], "1", "[1, 2]: second property");
assert.sameValue(subject[2], undefined, "[1, 2]: third property");
assert.sameValue(subject[3], undefined, "[1, 2]: fourth property");

subject = props([1, 2, 3]);
assert.sameValue(subject.length, 3, "[1, 2, 3]: length");
assert.sameValue(subject[0], "0", "[1, 2, 3]: first property");
assert.sameValue(subject[1], "1", "[1, 2, 3]: second property");
assert.sameValue(subject[2], "2", "[1, 2, 3]: third property");
assert.sameValue(subject[3], undefined, "[1, 2, 3]: fourth property");
