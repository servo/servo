// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    Empty arrow function returns undefined
---*/

var empty = () => {};
assert.sameValue(empty(), undefined);
