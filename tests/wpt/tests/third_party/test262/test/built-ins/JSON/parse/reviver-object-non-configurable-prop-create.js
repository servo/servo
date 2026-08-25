// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-internalizejsonproperty
description: >
  [[DefineOwnProperty]] validates property descriptor before applying.
  If [[DefineOwnProperty]] is unsuccessful, no exception is thrown.
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
    c. Else,
      i. Let keys be ? EnumerableOwnPropertyNames(val, "key").
      ii. For each String P in keys, do
        1. Let newElement be ? InternalizeJSONProperty(val, P).
        2. If newElement is undefined, then
          [...]
        3. Else,
          a. Perform ? CreateDataProperty(val, P, newElement).

  CreateDataProperty ( O, P, V )

  [...]
  4. Return ? O.[[DefineOwnProperty]](P, newDesc).
---*/

var json = '{"a": 1, "b": 2}';
var obj = JSON.parse(json, function(key, value) {
  if (key === 'a') {
    Object.defineProperty(this, 'b', {configurable: false});
  }
  if (key === 'b') return 22;

  return value;
});

assert.sameValue(obj.a, 1);
assert.sameValue(obj.b, 2);
