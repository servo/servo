// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack-constructor
description: >
  The AsyncDisposableStack constructor is the %AsyncDisposableStack% intrinsic object and the initial
  value of the AsyncDisposableStack property of the global object.
features: [explicit-resource-management]
---*/

assert.sameValue(
  typeof AsyncDisposableStack, 'function',
  'typeof AsyncDisposableStack is function'
);
