// Copyright (c) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
  delete operations throws TypeError exception when base object is unresolvable reference.
---*/

assert.throws(TypeError, () => {
  delete Object[0][0];
});
