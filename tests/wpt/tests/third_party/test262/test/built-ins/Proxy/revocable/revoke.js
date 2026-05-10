// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.2.2.1
description: >
    The returned object has a `revoked` property which is a function
info: |
    Proxy.revocable ( target, handler )

    7. Perform CreateDataProperty(result, "revoke", revoker).
features: [Proxy]
---*/

var r = Proxy.revocable({}, {});

assert.sameValue(typeof r.revoke, "function");
