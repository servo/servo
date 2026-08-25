// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
    Array.from uses a constructor other than Array.
esid: sec-array.from
es6id: 22.1.2.1
---*/

assert.sameValue(
  Array.from.call(Object, []).constructor,
  Object,
  'The value of Array.from.call(Object, []).constructor is expected to equal the value of Object'
);
