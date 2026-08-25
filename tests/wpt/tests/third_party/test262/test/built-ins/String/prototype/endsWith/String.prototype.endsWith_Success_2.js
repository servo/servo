// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: >
    endsWith should return true when called on 'word' and passed 'd'
    and with an endPosition of 4.
features: [String.prototype.endsWith]
---*/

assert.sameValue('word'.endsWith('d', 4), true, '"word".endsWith("d", 4)');
