// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-get-p-receiver
description: >
  If "get" trap is null or undefined, [[Get]] call is properly
  forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Get]] ( P, Receiver )

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "get").
  7. If trap is undefined, then
    a. Return ? target.[[Get]](P, Receiver).
features: [Proxy, Symbol]
---*/

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {
  get: null,
});

assert.sameValue(stringProxy.length, 3);
assert.sameValue(stringProxy[0], "s");
assert.sameValue(stringProxy[4], undefined);


var sym = Symbol();
var target = new Proxy({}, {
  get: function(_target, key) {
    switch (key) {
      case sym: return 1;
      case "10": return 2;
      case "foo": return 3;
    }
  },
});

var proxy = new Proxy(target, {
  get: null,
});

assert.sameValue(proxy[sym], 1);
assert.sameValue(proxy[10], 2);
assert.sameValue(Object.create(proxy).foo, 3);
assert.sameValue(proxy.bar, undefined);
