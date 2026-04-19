// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strictness of direct eval is not dependent on strictness of caller
esid: sec-strict-mode-code
info: |
    Eval code is strict mode code if it begins with a Directive Prologue that
    contains a Use Strict Directive or if the call to eval is a direct eval
    that is contained in strict mode code.
flags: [onlyStrict]
---*/

var count = 0;

(0,eval)('var static; count += 1;');

assert.sameValue(count, 1);

(0,eval)('with ({}) {} count += 1;');

assert.sameValue(count, 2);

(0,eval)('unresolvable = null; count += 1;');

assert.sameValue(count, 3);
