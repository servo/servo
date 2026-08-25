// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.2.2.1.1
description: >
    Calling the revoked function returns undefined
info: |
    Proxy Revocation Functions

    ...
    7. Return undefined.
features: [Proxy]
---*/

var r = Proxy.revocable({}, {});

assert.sameValue(r.revoke(), undefined);
