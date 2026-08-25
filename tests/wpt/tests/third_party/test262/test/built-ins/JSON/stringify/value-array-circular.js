// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonarray
description: >
  Circular array value throws a TypeError.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    a. Let isArray be ? IsArray(value).
    b. If isArray is true, return ? SerializeJSONArray(value).

  SerializeJSONArray ( value )

  1. If stack contains value, throw a TypeError exception because the structure is cyclical.
---*/

var direct = [];
direct.push(direct);

assert.throws(TypeError, function() {
  JSON.stringify(direct);
});

var indirect = [];
indirect.push([[indirect]]);

assert.throws(TypeError, function() {
  JSON.stringify(indirect);
});
