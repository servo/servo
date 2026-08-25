// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    rest index
---*/
assert.sameValue(
    (function(...args) { return args.length; })(1,2,3,4,5),
    5,
    "`(function(...args) { return args.length; })(1,2,3,4,5)` returns `5`"
);
assert.sameValue(
    (function(a, ...args) { return args.length; })(1,2,3,4,5),
    4,
    "`(function(a, ...args) { return args.length; })(1,2,3,4,5)` returns `4`"
);
assert.sameValue(
    (function(a, b, ...args) { return args.length; })(1,2,3,4,5),
    3,
    "`(function(a, b, ...args) { return args.length; })(1,2,3,4,5)` returns `3`"
);
assert.sameValue(
    (function(a, b, c, ...args) { return args.length; })(1,2,3,4,5),
    2,
    "`(function(a, b, c, ...args) { return args.length; })(1,2,3,4,5)` returns `2`"
);
assert.sameValue(
    (function(a, b, c, d, ...args) { return args.length; })(1,2,3,4,5),
    1,
    "`(function(a, b, c, d, ...args) { return args.length; })(1,2,3,4,5)` returns `1`"
);
assert.sameValue(
    (function(a, b, c, d, e, ...args) { return args.length; })(1,2,3,4,5),
    0,
    "`(function(a, b, c, d, e, ...args) { return args.length; })(1,2,3,4,5)` returns `0`"
);
