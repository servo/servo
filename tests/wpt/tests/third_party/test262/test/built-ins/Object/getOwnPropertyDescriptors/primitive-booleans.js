// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors accepts boolean primitives.
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

var trueResult = Object.getOwnPropertyDescriptors(true);

assert.sameValue(Object.keys(trueResult).length, 0, 'trueResult has 0 items');

var falseResult = Object.getOwnPropertyDescriptors(false);

assert.sameValue(Object.keys(falseResult).length, 0, 'falseResult has 0 items');
