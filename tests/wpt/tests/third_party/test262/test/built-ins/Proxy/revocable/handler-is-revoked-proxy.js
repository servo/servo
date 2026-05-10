// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxycreate
description: >
  A Proxy is created with its [[ProxyHandler]] as revoked Proxy.
info: |
  ProxyCreate ( target, handler )

  [...]
  3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
  [...]
  7. Set P.[[ProxyHandler]] to handler.
  8. Return P.
features: [Proxy]
---*/

var revocableHandler = Proxy.revocable({}, {});
revocableHandler.revoke();

var revocable = Proxy.revocable({}, revocableHandler.proxy);
assert.sameValue(typeof revocable.proxy, "object");
