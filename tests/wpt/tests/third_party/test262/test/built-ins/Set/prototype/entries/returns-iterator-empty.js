// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.entries
description: >
    Set.prototype.entries ( )

    ...
    2. Return CreateSetIterator(S, "key+value").


    23.2.5.1 CreateSetIterator Abstract Operation

    ...
    7. Return iterator.


---*/

var set = new Set();
var iterator = set.entries();
var result = iterator.next();

assert.sameValue(result.value, undefined, "The value of `result.value` is `undefined`");
assert.sameValue(result.done, true, "The value of `result.done` is `true`");
