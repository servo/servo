// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Object.getOwnPropertyDescriptors should not have its behavior impacted by modifications to Object.getOwnPropertyDescriptor
esid: sec-object.getownpropertydescriptors
author: Jordan Harband
---*/

function fakeObjectGetOwnPropertyDescriptor() {
  throw new Test262Error('The overriden version of Object.getOwnPropertyDescriptor was called!');
}
Object.getOwnPropertyDescriptor = fakeObjectGetOwnPropertyDescriptor;

assert.sameValue(
  Object.getOwnPropertyDescriptor,
  fakeObjectGetOwnPropertyDescriptor,
  'Sanity check failed: could not modify the global Object.getOwnPropertyDescriptor'
);

assert.sameValue(Object.keys(Object.getOwnPropertyDescriptors({
  a: 1
})).length, 1, 'Expected object with 1 key to have 1 descriptor');
