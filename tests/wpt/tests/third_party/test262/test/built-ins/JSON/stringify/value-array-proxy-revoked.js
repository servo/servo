// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Revoked array proxy value produces a TypeError.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  12. Return ? SerializeJSONProperty(the empty String, wrapper).

  SerializeJSONProperty ( key, holder )

  [...]
  10. If Type(value) is Object and IsCallable(value) is false, then
    a. Let isArray be ? IsArray(value).

  IsArray ( argument )

  [...]
  3. If argument is a Proxy exotic object, then
    a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
features: [Proxy]
---*/

var handle = Proxy.revocable([], {});

handle.revoke();

assert.throws(TypeError, function() {
  JSON.stringify(handle.proxy);
}, 'top-level value');

assert.throws(TypeError, function() {
  JSON.stringify([[[handle.proxy]]]);
}, 'nested value');
