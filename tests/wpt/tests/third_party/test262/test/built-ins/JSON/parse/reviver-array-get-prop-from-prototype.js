// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-internalizejsonproperty
description: >
  `name` property is obtained with [[Get]] from prototype chain.
info: |
  JSON.parse ( text [ , reviver ] )

  [...]
  7. If IsCallable(reviver) is true, then
    [...]
    d. Return ? InternalizeJSONProperty(root, rootName).

  InternalizeJSONProperty ( holder, name )

  1. Let val be ? Get(holder, name).
  2. If Type(val) is Object, then
    a. Let isArray be ? IsArray(val).
    b. If isArray is true, then
      [...]
      iii. Repeat, while I < len,
        1. Let newElement be ? InternalizeJSONProperty(val, ! ToString(I)).
        2. If newElement is undefined, then
          [...]
        3. Else,
          a. Perform ? CreateDataProperty(val, ! ToString(I), newElement).
---*/

Array.prototype[1] = 3;

var json = '[1, 2]';
var arr = JSON.parse(json, function(key, value) {
  if (key === '0') {
    assert(delete this[1]);
  }

  return value;
});

assert(delete Array.prototype[1]);
assert.sameValue(arr[0], 1);
assert(arr.hasOwnProperty('1'));
assert.sameValue(arr[1], 3);
