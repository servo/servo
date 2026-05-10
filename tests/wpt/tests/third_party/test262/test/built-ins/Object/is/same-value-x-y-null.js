// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.10
description: >
    Object.is ( value1, value2 )

    7.2.9 SameValue(x, y)
    
    ...
    3. If Type(x) is different from Type(y), return false.
    ...
    5. If Type(x) is Null, return true.
    ...

---*/

assert.sameValue(Object.is(null, null), true, "`Object.is(null, null)` returns `true`");
