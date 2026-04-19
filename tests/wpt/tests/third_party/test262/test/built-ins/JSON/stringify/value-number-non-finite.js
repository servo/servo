// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Non-finite numbers as serialized as null.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  4. If Type(value) is Object, then
    a. If value has a [[NumberData]] internal slot, then
      i. Set value to ? ToNumber(value).
  [...]
  9. If Type(value) is Number, then
    a. If value is finite, return ! ToString(value).
    b. Return "null".
---*/

assert.sameValue(JSON.stringify(Infinity), 'null');
assert.sameValue(JSON.stringify({key: -Infinity}), '{"key":null}');
assert.sameValue(JSON.stringify([NaN]), '[null]');
