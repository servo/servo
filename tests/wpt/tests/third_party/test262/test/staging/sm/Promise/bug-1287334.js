// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var promise = Promise.resolve(1);
var FakeCtor = function(exec){ exec(function(){}, function(){}); };
Object.defineProperty(Promise, Symbol.species, {value: FakeCtor});
// This just shouldn't crash. It does without bug 1287334 fixed.
promise.then(function(){});
