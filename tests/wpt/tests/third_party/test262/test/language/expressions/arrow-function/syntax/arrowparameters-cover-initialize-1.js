// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2.1
description: >
    ArrowParameters[Yield] :
      ...
      CoverParenthesizedExpressionAndArrowParameterList[?Yield]

---*/
var af = (x = 1) => x;


assert.sameValue(typeof af, "function");
assert.sameValue(af(), 1);
assert.sameValue(af(2), 2);
