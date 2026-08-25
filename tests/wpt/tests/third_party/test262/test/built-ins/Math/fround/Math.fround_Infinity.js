// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.2.2.17
author: Ryan Lewis
description: Math.fround should return Infinity if called with Infinity.
---*/

assert.sameValue(Math.fround(Infinity), Infinity, 'Math.fround(Infinity)');
