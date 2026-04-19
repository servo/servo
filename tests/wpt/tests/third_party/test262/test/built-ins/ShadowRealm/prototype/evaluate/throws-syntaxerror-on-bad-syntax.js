// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate throws a SyntaxError if the syntax can't be parsed
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(SyntaxError, () => r.evaluate('...'), 'SyntaxError exposed to Parent');
assert.throws(SyntaxError, () => r.evaluate(`
  "use strict";
  throw "do not evaluate";
  function lol(){
    arguments = 1;
  }
`), 'Strict mode only SyntaxError, setting value to a fn arguments');
assert.throws(SyntaxError, () => r.evaluate(`
  "use strict";
  throw "do not evaluate";
  var public = 1;
`), 'Strict mode only SyntaxError, var public');
