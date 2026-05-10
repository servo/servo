// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: >
    String should return false if a letter is not found in the word
    starting from the passed location.
features: [String.prototype.includes]
---*/

assert.sameValue('word'.includes('o', 3), false, '"word".includes("o", 3)');
