// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Allows symbol keys.
esid: sec-object.fromentries
features: [Symbol, Object.fromEntries]
---*/

var key = Symbol();
var result = Object.fromEntries([[key, 'value']]);
assert.sameValue(result[key], 'value');
