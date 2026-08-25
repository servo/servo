// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonobject
description: >
  Circular object value (returned from replacer function) throws a TypeError.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    [...]
    c. Return ? SerializeJSONObject(value).

  SerializeJSONObject ( value )

  1. If stack contains value, throw a TypeError exception because the structure is cyclical.
---*/

var direct = {prop: {}};
var directReplacer = function(k, v) {
  return direct;
};

assert.throws(TypeError, function() {
  JSON.stringify(direct, directReplacer);
});

var indirect = {p1: {p2: {}}};
var indirectReplacer = function(key, value) {
  if (key === 'p2') {
    return indirect;
  } 

  return value;
};

assert.throws(TypeError, function() {
  JSON.stringify(indirect, indirectReplacer);
});
