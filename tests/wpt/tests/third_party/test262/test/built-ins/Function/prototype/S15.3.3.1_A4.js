// Copyright 2011 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Detects whether the value of a function's "prototype" property
    as seen by normal object operations might deviate from the value
    as seem by Object.getOwnPropertyDescriptor
es5id: 15.3.3.1_A4
description: >
    Checks if reading a function's .prototype directly  agrees with
    reading it via Object.getOwnPropertyDescriptor, after  having set
    it by Object.defineProperty.
---*/

function foo() {}

Object.defineProperty(foo, 'prototype', {
  value: {}
});

assert.sameValue(
  foo.prototype,
  Object.getOwnPropertyDescriptor(foo, 'prototype').value,
  'The value of foo.prototype is expected to equal the value of Object.getOwnPropertyDescriptor(foo, \'prototype\').value'
);
