// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-UnaryExpression
description: >
  While increments and decrements are restricted to use with NewTarget,
  other unary operators should not throw SyntaxError.
info: |
  UnaryExpression[Yield, Await]:
    UpdateExpression[?Yield, ?Await]:
      LeftHandSideExpression[?Yield, ?Await]:
        NewExpression[?Yield, ?Await]:
          MemberExpression[Yield, Await]:
            MetaProperty:
              NewTarget
features: [new.target, async-functions]
flags: [async]
includes: [asyncHelpers.js]
---*/

(function() { assert.sameValue(delete (new.target), true); })();
(function() { assert.sameValue(void new.target, undefined); })();
new function() { assert.sameValue(typeof new.target, 'function'); };
new function() { assert.sameValue(+(new.target), NaN); };
(function() { assert.sameValue(-(new.target), NaN); })();
new function() { assert.sameValue(~new.target, -1); };
(function() { assert.sameValue(!new.target, true); })();
new function() { assert.sameValue(delete void typeof +-~!(new.target), true); };

asyncTest(async function() {
  assert.sameValue(await new.target, undefined);
});
