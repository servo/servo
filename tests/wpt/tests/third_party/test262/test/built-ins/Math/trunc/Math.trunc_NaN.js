// Copyright (c) 2014 Ryan Lewis. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.2.2.35
author: Ryan Lewis
description: Math.trunc should return NaN when called with NaN.
---*/

assert.sameValue(Math.trunc(NaN), NaN);
