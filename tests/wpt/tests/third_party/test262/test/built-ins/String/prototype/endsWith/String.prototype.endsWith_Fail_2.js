// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: >
    endsWith should return false when called on 'word' and passed 'd',
    with an endPosition of 3.
features: [String.prototype.endsWith]
---*/

assert.sameValue('word'.endsWith('d', 3), false, '"word".endsWith("d", 3)');
