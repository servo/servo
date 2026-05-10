// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Revoked proxy of array as replacer produces a TypeError
  (honoring the realm of the current execution context).
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
     a. If IsCallable(replacer) is true, then
        i. Let ReplacerFunction be replacer.
     b. Else,
        i. Let isArray be ? IsArray(replacer).

  IsArray ( argument )

  [...]
  3. If argument is a Proxy exotic object, then
     a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
     b. Let target be argument.[[ProxyTarget]].
     c. Return ? IsArray(target).
features: [cross-realm, Proxy]
---*/

var OProxy = $262.createRealm().global.Proxy;
var handle = OProxy.revocable([], {});

handle.revoke();

assert.throws(TypeError, function() {
  JSON.stringify({}, handle.proxy);
});
