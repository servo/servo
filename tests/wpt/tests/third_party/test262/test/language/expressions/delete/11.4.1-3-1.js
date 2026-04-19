// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
    delete operator returns true when deleting an unresolvable
    reference
flags: [noStrict]
---*/

assert.sameValue(delete unresolvable, true, 'delete unresolvable === true');
