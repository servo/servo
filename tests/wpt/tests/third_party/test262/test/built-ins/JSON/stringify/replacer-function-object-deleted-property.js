// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Replacer function is called on properties, deleted during stringification.
info: |
  SerializeJSONObject ( value )

  [...]
  5. If PropertyList is not undefined, then
    [...]
  6. Else,
    a. Let K be ? EnumerableOwnPropertyNames(value, key).
  [...]
  8. For each element P of K, do
    a. Let strP be ? SerializeJSONProperty(P, value).
    [...]

  SerializeJSONProperty ( key, holder )

  1. Let value be ? Get(holder, key).
  [...]
  3. If ReplacerFunction is not undefined, then
    a. Set value to ? Call(ReplacerFunction, holder, « key, value »).
---*/

var obj = {
  get a() {
    delete this.b;
    return 1;
  },
  b: 2,
};

var replacer = function(key, value) {
  if (key === 'b') {
    assert.sameValue(value, undefined);
    return '<replaced>';
  }

  return value;
};

assert.sameValue(
  JSON.stringify(obj, replacer),
  '{"a":1,"b":"<replaced>"}'
);
