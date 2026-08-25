// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: >
    String should return true when called on 'word' and passed 'w' and
    the location 0.
features: [String.prototype.includes]
---*/

assert.sameValue('word'.includes('w', 0), true, '"word".includes("w", 0)');
