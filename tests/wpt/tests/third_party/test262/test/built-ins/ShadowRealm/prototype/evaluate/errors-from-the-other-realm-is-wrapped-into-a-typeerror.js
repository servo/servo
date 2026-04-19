// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wraps errors from other realm into TypeErrors
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.throws(SyntaxError, () => r.evaluate('...'), 'SyntaxError exposed to Parent');
assert.throws(TypeError, () => r.evaluate('throw 42'), 'throw primitive => TypeError');
assert.throws(TypeError, () => r.evaluate('throw new ReferenceError("aaa")'), 'custom ctor => TypeError');
assert.throws(TypeError, () => r.evaluate('throw new TypeError("aaa")'), 'Child TypeError => Parent TypeError');
assert.throws(TypeError, () => r.evaluate('eval("...");'), 'syntaxerror parsing coming after runtime evaluation');
assert.throws(TypeError, () => r.evaluate(`
  'use strict';
  eval("var public = 1;");
`), 'strict-mode only syntaxerror parsing coming after runtime evaluation');
