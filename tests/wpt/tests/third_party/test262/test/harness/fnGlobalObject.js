// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including fnGlobalObject.js will expose a function:

        fnGlobalObject

    fnGlobalObject returns a reference to the global object.

includes: [fnGlobalObject.js]
---*/

var gO = fnGlobalObject();

assert(typeof gO === "object");
assert.sameValue(gO, this);
