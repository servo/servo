// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator throws ReferenceError when deleting an explicitly
    qualified yet unresolvable reference (base obj undefined)
---*/

assert.throws(ReferenceError, function() {
  delete unresolvable.x;
});
