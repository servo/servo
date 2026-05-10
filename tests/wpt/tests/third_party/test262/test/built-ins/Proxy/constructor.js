// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.2.1
description: >
    The Proxy constructor is the %Proxy% intrinsic object and the
    initial value of the Proxy property of the global object.
features: [Proxy]
---*/

assert.sameValue(typeof Proxy, "function", "`typeof Proxy` is `'function'`");
