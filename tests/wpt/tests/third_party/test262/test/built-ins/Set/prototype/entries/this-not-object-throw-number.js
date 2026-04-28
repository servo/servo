// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.entries
description: >
    Set.prototype.entries ( )

    ...
    2. Return CreateSetIterator(S, "key+value").


    23.2.5.1 CreateSetIterator Abstract Operation

    1. If Type(set) is not Object, throw a TypeError exception.
    ...
---*/

assert.throws(TypeError, function() {
  Set.prototype.entries.call(0);
});

assert.throws(TypeError, function() {
  var s = new Set();
  s.entries.call(0);
});
