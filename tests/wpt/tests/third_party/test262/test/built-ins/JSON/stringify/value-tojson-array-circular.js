// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonarray
description: >
  Circular array value (returned from toJSON method) throws a TypeError.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  2. If Type(value) is Object, then
    a. Let toJSON be ? Get(value, "toJSON").
    b. If IsCallable(toJSON) is true, then
      i. Set value to ? Call(toJSON, value, « key »).
  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    a. Let isArray be ? IsArray(value).
    b. If isArray is true, return ? SerializeJSONArray(value).

  SerializeJSONArray ( value )

  1. If stack contains value, throw a TypeError exception because the structure is cyclical.
---*/

var arr = [];
var circular = [arr];

arr.toJSON = function() {
  return circular;
};

assert.throws(TypeError, function() {
  JSON.stringify(circular);
});
