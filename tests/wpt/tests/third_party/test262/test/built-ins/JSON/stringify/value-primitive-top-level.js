// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonproperty
description: >
  Top-level primitive values are stringified correctly.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  5. If value is null, return "null".
  6. If value is true, return "true".
  7. If value is false, return "false".
  8. If Type(value) is String, return QuoteJSONString(value).
  9. If Type(value) is Number, then
    a. If value is finite, return ! ToString(value).
  [...]
  11. Return undefined.
---*/

assert.sameValue(JSON.stringify(null), 'null');
assert.sameValue(JSON.stringify(true), 'true');
assert.sameValue(JSON.stringify(false), 'false');
assert.sameValue(JSON.stringify('str'), '"str"');
assert.sameValue(JSON.stringify(123), '123');
assert.sameValue(JSON.stringify(undefined), undefined);
