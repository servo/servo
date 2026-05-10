// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxycreate
description: >
  A Proxy is created with its [[ProxyTarget]] as revoked function Proxy.
info: |
  ProxyCreate ( target, handler )

  [...]
  3. Let P be ! MakeBasicObject(« [[ProxyHandler]], [[ProxyTarget]] »).
  [...]
  6. Set P.[[ProxyTarget]] to target.
  [...]
  8. Return P.
features: [Proxy]
---*/

var revocableTarget = Proxy.revocable(function() {}, {});
revocableTarget.revoke();

var revocable = Proxy.revocable(revocableTarget.proxy, {});
assert.sameValue(typeof revocable.proxy, "function");
