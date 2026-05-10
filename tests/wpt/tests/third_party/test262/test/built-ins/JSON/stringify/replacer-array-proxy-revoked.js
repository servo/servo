// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: Revoked proxy value produces a TypeError.
info: |
  [...]
  4. If Type(replacer) is Object, then
     a. If IsCallable(replacer) is true, then
        i. Let ReplacerFunction be replacer.
     b. Else,
        i. Let isArray be ? IsArray(replacer).

  7.2.2 IsArray

  [...]
  3. If argument is a Proxy exotic object, then
     a. If the value of the [[ProxyHandler]] internal slot of argument is null,
        throw a TypeError exception.
     b. Let target be the value of the [[ProxyTarget]] internal slot of
        argument.
     c. Return ? IsArray(target).
features: [Proxy]
---*/

var handle = Proxy.revocable([], {});

handle.revoke();

assert.throws(TypeError, function() {
  JSON.stringify({}, handle.proxy);
});
