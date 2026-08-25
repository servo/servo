// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Negative zero numbers are stringified to "0".
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  9. If Type(value) is Number, then
    a. If value is finite, return ! ToString(value).

  NumberToString ( m )

  [...]
  2. If m is +0 or -0, return the String "0".
---*/

assert.sameValue(JSON.stringify(-0), '0');
assert.sameValue(JSON.stringify(['-0', 0, -0]), '["-0",0,0]');
assert.sameValue(JSON.stringify({key: -0}), '{"key":0}');
