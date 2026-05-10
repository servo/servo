// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
    If literalSegments ≤ 0, return the empty string.
---*/

assert.sameValue(String.raw``, '');
