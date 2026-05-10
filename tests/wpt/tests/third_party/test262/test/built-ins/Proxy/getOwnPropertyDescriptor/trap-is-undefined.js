// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.5
description: >
    Return target.[[GetOwnProperty]](P) if trap is undefined.
info: |
    [[GetOwnProperty]] (P)

    ...
    8. If trap is undefined, then
        a. Return target.[[GetOwnProperty]](P).
    ...
includes: [propertyHelper.js]
features: [Proxy]
---*/

var target = {
  attr: 1
};
var p = new Proxy(target, {});

var proxyDesc = Object.getOwnPropertyDescriptor(p, "attr");

verifyEqualTo(p, "attr", 1);
verifyProperty(p, "attr", {
  writable: true,
  enumerable: true,
  configurable: true,
});
