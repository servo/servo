// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonobject
description: >
  Circular object value throws a TypeError.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    [...]
    c. Return ? SerializeJSONObject(value).

  SerializeJSONObject ( value )

  1. If stack contains value, throw a TypeError exception because the structure is cyclical.
---*/

var direct = {};
direct.prop = direct;

assert.throws(TypeError, function() {
  JSON.stringify(direct);
});

var indirect = {
  p1: {
    p2: {
      get p3() {
        return indirect;
      },
    },
  },
};

assert.throws(TypeError, function() {
  JSON.stringify(indirect);
});
