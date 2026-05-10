// Copyright 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 8.1.1.4.1
description: >
    `Symbol.unscopables` is not referenced when finding bindings in global scope
info: |
    1. Let envRec be the global Environment Record for which the method was
       invoked.
    2. Let DclRec be envRec.[[DeclarativeRecord]].
    3. If DclRec.HasBinding(N) is true, return true.
    4. Let ObjRec be envRec.[[ObjectRecord]].
    5. Return ObjRec.HasBinding(N).
features: [Symbol.unscopables]
---*/

var x = 86;
this[Symbol.unscopables] = {
  x: true
};
assert.sameValue(x, 86);
