// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.2.2.18
author: Ryan Lewis
description: >
    Math.hypot should return Infinity if called with any argument that
    is Infinity.
---*/

assert.sameValue(Math.hypot(NaN, Infinity), Infinity, 'Math.hypot(NaN, Infinity)');
