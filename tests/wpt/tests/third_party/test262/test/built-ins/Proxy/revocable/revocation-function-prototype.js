// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 26.2.2.1.1
description: The [[Prototype]] of Proxy Revocation functions
info: |
  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified every built-in function and every built-in
    constructor has the Function prototype object, which is the initial
    value of the expression Function.prototype (19.2.3), as the value of
    its [[Prototype]] internal slot.
features: [Proxy]
---*/

var revocationFunction = Proxy.revocable({}, {}).revoke;

assert.sameValue(Object.getPrototypeOf(revocationFunction), Function.prototype);
