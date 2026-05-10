// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-other-properties-of-the-global-object-globalthis
description: "'globalThis' should be the global object"
author: Jordan Harband
features: [globalThis]
---*/

assert.sameValue(this, globalThis);
assert.sameValue(globalThis.globalThis, globalThis);

assert.sameValue(Array, globalThis.Array);
assert.sameValue(Boolean, globalThis.Boolean);
assert.sameValue(Date, globalThis.Date);
assert.sameValue(Error, globalThis.Error);
assert.sameValue(Function, globalThis.Function);
assert.sameValue(JSON, globalThis.JSON);
assert.sameValue(Math, globalThis.Math);
assert.sameValue(Number, globalThis.Number);
assert.sameValue(RegExp, globalThis.RegExp);
assert.sameValue(String, globalThis.String);

var globalVariable = {};
assert.sameValue(globalVariable, globalThis.globalVariable);
