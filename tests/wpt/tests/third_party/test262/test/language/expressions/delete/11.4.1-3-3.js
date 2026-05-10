// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns true when deleting an explicitly qualified
    yet unresolvable reference (property undefined for base obj)
---*/

var o = {};
assert.sameValue(delete o.x, true);
