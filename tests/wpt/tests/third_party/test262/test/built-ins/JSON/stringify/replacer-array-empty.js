// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonobject
description: >
  Objects are serialized to {} if replacer array is empty.
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

assert.sameValue(
  JSON.stringify({a: 1, b: 2}, []),
  '{}'
);

assert.sameValue(
  JSON.stringify({a: 1, b: {c: 2}}, []),
  '{}'
);

assert.sameValue(
  JSON.stringify([1, {a: 2}], []),
  '[1,{}]'
);
