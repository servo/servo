// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set.prototype.values
description: >
    Returns an iterator that's already done if Set is empty.
---*/

var set = new Set();
var iterator = set.values();
var result = iterator.next();
assert.sameValue(result.value, undefined, "The value of `result.value` is `undefined`");
assert.sameValue(result.done, true, "The value of `result.done` is `true`");
