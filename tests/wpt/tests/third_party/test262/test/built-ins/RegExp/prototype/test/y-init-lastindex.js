// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Honors initial value of the `lastIndex` property
es6id: 21.2.5.13
info: |
    21.2.5.13 RegExp.prototype.test( S )

    [...]
    5. Let match be RegExpExec(R, string).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    4. Let lastIndex be ToLength(Get(R,"lastIndex")).
    [...]
    8. Let sticky be ToBoolean(Get(R, "sticky")).
    9. ReturnIfAbrupt(sticky).
    10. If global is false and sticky is false, let lastIndex be 0.
---*/

var r = /./y;
r.lastIndex = 1;

assert.sameValue(r.test('a'), false);
