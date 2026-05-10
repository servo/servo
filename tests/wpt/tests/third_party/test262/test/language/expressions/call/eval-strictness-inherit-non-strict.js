// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Evaluated code honors the strictness of the calling context
esid: sec-function-calls-runtime-semantics-evaluation
info: |
    [...]
    3. If Type(ref) is Reference and IsPropertyReference(ref) is false and
       GetReferencedName(ref) is "eval", then
       a. If SameValue(func, %eval%) is true, then
          [...]
          iv. If the source code matching this CallExpression is strict code,
              let strictCaller be true. Otherwise let strictCaller be false.
          [...]
flags: [noStrict]
---*/

var count = 0;

eval('var static; count += 1;');

assert.sameValue(count, 1);

eval('with ({}) {} count += 1;');

assert.sameValue(count, 2);

eval('unresolvable = null; count += 1;');

assert.sameValue(count, 3);
