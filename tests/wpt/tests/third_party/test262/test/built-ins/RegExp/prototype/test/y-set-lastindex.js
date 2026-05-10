// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Sets the `lastIndex` property to the end index of the first match
es6id: 21.2.5.13
info: |
    21.2.5.13 RegExp.prototype.test( S )

    [...]
    5. Let match be RegExpExec(R, string).

    21.2.5.2.2 Runtime Semantics: RegExpBuiltinExec ( R, S )

    [...]
    8. Let sticky be ToBoolean(Get(R, "sticky")).
    [...]
    18. If global is true or sticky is true,
        a. Let setStatus be Set(R, "lastIndex", e, true).
---*/

var r = /abc/y;

r.test('abc');

assert.sameValue(r.lastIndex, 3);
