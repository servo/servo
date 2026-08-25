// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack-constructor
description: >
  The DisposableStack constructor is the %DisposableStack% intrinsic object and the initial
  value of the DisposableStack property of the global object.
features: [explicit-resource-management]
---*/

assert.sameValue(
  typeof DisposableStack, 'function',
  'typeof DisposableStack is function'
);
