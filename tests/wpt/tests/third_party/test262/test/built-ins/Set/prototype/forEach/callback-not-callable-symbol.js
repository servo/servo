// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.foreach
description: >
    Set.prototype.forEach ( callbackfn [ , thisArg ] )

    ...
    4. If IsCallable(callbackfn) is false, throw a TypeError exception.
    ...

    Passing `symbol` as callback

features: [Symbol]
---*/

var s = new Set([1]);

assert.throws(TypeError, function() {
  s.forEach(Symbol());
});
