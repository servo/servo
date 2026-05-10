// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Created objects inherit from Object.prototype.
esid: sec-object.fromentries
features: [Object.fromEntries]
---*/

var result = Object.fromEntries([]);
assert.sameValue(Object.getPrototypeOf(result), Object.prototype);
