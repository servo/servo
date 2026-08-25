// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.3
description: >
    Return target.[[IsExtensible]]() if trap is undefined.
info: |
    [[IsExtensible]] ( )

    ...
    7. If trap is undefined, then
        a. Return target.[[IsExtensible]]().
    ...
features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {});

assert.sameValue(Object.isExtensible(p), true);

Object.preventExtensions(target);

assert.sameValue(Object.isExtensible(p), false);
