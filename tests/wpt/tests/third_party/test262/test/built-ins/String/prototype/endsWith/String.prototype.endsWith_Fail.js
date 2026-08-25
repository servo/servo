// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Ryan Lewis
description: endsWith should return false when called on 'word' and passed 'r'.
features: [String.prototype.endsWith]
---*/

assert.sameValue('word'.endsWith('r'), false, '"word".endsWith("r")');
