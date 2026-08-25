// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.2.2.35
author: Ryan Lewis
description: Math.trunc should return 4578 if called with 4578.584949
---*/

assert.sameValue(Math.trunc(4578.584949), 4578, 'Math.trunc(4578.584949)');
