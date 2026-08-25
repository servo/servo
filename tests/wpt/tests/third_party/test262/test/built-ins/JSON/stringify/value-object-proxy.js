// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-serializejsonobject
description: >
  Proxy of an object is treated as regular object.
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
  6. Else,
    a. Let K be ? EnumerableOwnPropertyNames(value, "key").
  7. Let partial be a new empty List.
  8. For each element P of K, do
    a. Let strP be ? SerializeJSONProperty(P, value).
features: [Proxy]
---*/

var objectProxy = new Proxy({}, {
  getOwnPropertyDescriptor: function() {
    return {value: 1, writable: true, enumerable: true, configurable: true};
  },
  get: function() {
    return 1;
  },
  ownKeys: function() {
    return ['a', 'b'];
  },
});

assert.sameValue(
  JSON.stringify(objectProxy), '{"a":1,"b":1}', 'proxy for an object'
);
assert.sameValue(
  JSON.stringify({l1: {l2: objectProxy}}),
  '{"l1":{"l2":{"a":1,"b":1}}}',
  'proxy for an object (nested)'
);

var objectProxyProxy = new Proxy(objectProxy, {});
assert.sameValue(
  JSON.stringify({l1: {l2: objectProxyProxy}}),
  '{"l1":{"l2":{"a":1,"b":1}}}',
  'proxy for a proxy for an object (nested)'
);
