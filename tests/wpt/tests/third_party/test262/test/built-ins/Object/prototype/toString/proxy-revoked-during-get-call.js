// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.tostring
description: >
  If Proxy is revoked during Get call, a string is returned.
info: |
  Object.prototype.toString ( )

  [...]
  4. Let isArray be ? IsArray(O).
  [...]
  14. Else, let builtinTag be "Object".
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".

  IsArray ( argument )

  [...]
  3. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
    a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
    b. Let target be argument.[[ProxyTarget]].
    c. Return ? IsArray(target).
features: [Proxy]
---*/

var handle1 = Proxy.revocable([], {
  get: function() {
    handle1.revoke();
  },
});

assert.sameValue(
  Object.prototype.toString.call(handle1.proxy),
  "[object Array]"
);


var handle2 = Proxy.revocable({}, {
  get: function() {
    handle2.revoke();
  },
});

var handle2Proxy = new Proxy(handle2.proxy, {});
assert.sameValue(
  Object.prototype.toString.call(handle2Proxy),
  "[object Object]"
);
