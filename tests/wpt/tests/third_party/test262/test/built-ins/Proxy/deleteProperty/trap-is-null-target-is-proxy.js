// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-delete-p
description: >
  If "deleteProperty" trap is null or undefined, [[Delete]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Delete]] ( P )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "deleteProperty").
  7. If trap is undefined, then
    a. Return ? target.[[Delete]](P).
features: [Proxy, Reflect]
---*/

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {
  deleteProperty: null,
});

assert(!Reflect.deleteProperty(stringProxy, "length"));
assert.throws(TypeError, function() {
  "use strict";
  delete stringProxy[0];
});


var regExpTarget = new Proxy(/(?:)/g, {});
var regExpProxy = new Proxy(regExpTarget, {
  deleteProperty: null,
});

assert(delete regExpProxy.foo);
assert.throws(TypeError, function() {
  "use strict";
  delete regExpProxy.lastIndex;
});
