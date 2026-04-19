// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%arrayiteratorprototype%-@@tostringtag
description: >
    `Object.prototype.toString` should honor the value of the @@toStringTag
    attribute.
es6id: 22.1.5.2.2
features: [Symbol.iterator]
---*/

var iter = [][Symbol.iterator]();

assert.sameValue("[object Array Iterator]", Object.prototype.toString.call(iter));
