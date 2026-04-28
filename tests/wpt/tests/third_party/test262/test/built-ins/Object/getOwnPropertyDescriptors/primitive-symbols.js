// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors accepts Symbol primitives.
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
features: [Symbol]
---*/

var result = Object.getOwnPropertyDescriptors(Symbol());

assert.sameValue(Object.keys(result).length, 0, 'symbol primitive has no descriptors');
