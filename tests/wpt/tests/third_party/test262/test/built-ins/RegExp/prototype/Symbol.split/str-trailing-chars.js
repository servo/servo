// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Characters after the final match are appended to the result
info: |
    [...]
    25. Let T be a String value equal to the substring of S consisting of the
        elements at indices p (inclusive) through size (exclusive).
    26. Assert: The following call will never result in an abrupt completion.
    27. Perform CreateDataProperty(A, ToString(lengthA), T ).
    28. Return A.
features: [Symbol.split]
---*/

var result = /d/[Symbol.split]('abcdefg');

assert(Array.isArray(result));
assert.sameValue(result.length, 2);
assert.sameValue(result[0], 'abc');
assert.sameValue(result[1], 'efg');
