// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonarray
description: >
  Proxy of an array is treated as an array.
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

  [...]
  6. Let len be ? LengthOfArrayLike(value).
  7. Let index be 0.
  8. Repeat, while index < len
    a. Let strP be ? SerializeJSONProperty(! ToString(index), value).
features: [Proxy]
---*/

var arrayProxy = new Proxy([], {
  get: function(_target, key) {
    if (key === 'length') return 2;
    return Number(key);
  },
});

assert.sameValue(
  JSON.stringify(arrayProxy), '[0,1]', 'proxy for an array'
);
assert.sameValue(
  JSON.stringify([[arrayProxy]]), '[[[0,1]]]', 'proxy for an array (nested)'
);

var arrayProxyProxy = new Proxy(arrayProxy, {});
assert.sameValue(
  JSON.stringify([[arrayProxyProxy]]),
  '[[[0,1]]]',
  'proxy for a proxy for an array (nested)'
);
