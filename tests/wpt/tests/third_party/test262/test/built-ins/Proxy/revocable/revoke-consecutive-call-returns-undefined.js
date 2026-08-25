// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.2.2.1.1
description: >
    Calling the revoked function again will return undefined
info: |
    Proxy Revocation Functions

    ...
    1. Let p be the value of F’s [[RevocableProxy]] internal slot.
    2. If p is null, return undefined.
features: [Proxy]
---*/

var r = Proxy.revocable({}, {});

r.revoke();

assert.sameValue(r.revoke(), undefined);
