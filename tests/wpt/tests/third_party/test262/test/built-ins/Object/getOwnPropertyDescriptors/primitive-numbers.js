// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Object.getOwnPropertyDescriptors accepts number primitives.
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(0)).length, 0, '0 has zero descriptors');
assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(-0)).length, 0, '-0 has zero descriptors');
assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(Infinity)).length, 0, 'Infinity has zero descriptors');
assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(-Infinity)).length, 0, '-Infinity has zero descriptors');
assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(NaN)).length, 0, 'NaN has zero descriptors');
assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors(Math.PI)).length, 0, 'Math.PI has zero descriptors');
