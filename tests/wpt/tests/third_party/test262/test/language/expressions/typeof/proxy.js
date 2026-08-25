// Copyright (C) 2019 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  The typeof operator on an proxy should match the typeof value the proxy wraps,
  even if the proxy is later revoked.
esid: sec-typeof-operator
info: |
  The typeof Operator

  Runtime Semantics: Evaluation

    ...
    Return a String according to Table 35.

  #table-35

  Object (does not implement [[Call]]) "object"
  Object (implements [[Call]]) "function"


  ProxyCreate ( target, handler )
    ...
    7. If IsCallable(target) is true, then
       a. Set P.[[Call]] as specified in 9.5.12.
    ...
features: [Proxy]
---*/

assert.sameValue(typeof new Proxy({}, {}), 'object');

assert.sameValue(typeof new Proxy(function() {}, {}), 'function');

const rp1 = Proxy.revocable({}, {});
rp1.revoke();
assert.sameValue(typeof rp1.proxy, 'object');

const rp2 = Proxy.revocable(function() {}, {});
rp2.revoke();
assert.sameValue(typeof rp2.proxy, 'function');
