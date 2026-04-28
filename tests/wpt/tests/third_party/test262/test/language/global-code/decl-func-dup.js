// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    redeclaration outermost:
    allowed to redeclare function declaration with function declaration
---*/
function f() { return 1; } function f() { return 2; }

assert.sameValue(f(), 2);
