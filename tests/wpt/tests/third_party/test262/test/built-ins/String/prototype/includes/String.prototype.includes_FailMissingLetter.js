// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: String should return false if a letter is not found in the word.
features: [String.prototype.includes]
---*/

assert.sameValue('word'.includes('a', 0), false, '"word".includes("a", 0)');
