// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonobject
description: >
  Keys order of serialized objects is determined by replacer array.
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

  [...]
  5. If PropertyList is not undefined, then
    a. Let K be PropertyList.
---*/

var replacer = ['c', 'b', 'a'];

assert.sameValue(
  JSON.stringify({b: 1, a: 2, c: 3}, replacer),
  '{"c":3,"b":1,"a":2}'
);

assert.sameValue(
  JSON.stringify({a: {b: 2, c: 3}}, replacer),
  '{"a":{"c":3,"b":2}}'
);
