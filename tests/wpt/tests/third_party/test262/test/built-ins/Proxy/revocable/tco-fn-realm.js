// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Realm of the TypeError from invoking a revoked Proxy during tail-call
  optimization
esid: sec-tail-position-calls
flags: [onlyStrict]
features: [Proxy, tail-call-optimization]
---*/

var other = $262.createRealm();
var F = other.evalScript(`
  (function() {
    var proxyObj = Proxy.revocable(function() {}, {});
    var proxy = proxyObj.proxy;
    var revoke = proxyObj.revoke;
    revoke();
    return proxy();
  })
`);

assert.throws(other.global.TypeError, function() {
  F();
});
