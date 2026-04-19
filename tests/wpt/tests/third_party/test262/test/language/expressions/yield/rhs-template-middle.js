// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions
es6id: 14.4
description: YieldExpression may be followed by a TemplateMiddle construct
info: |
  The syntactic context immediately following yield requires use of the
  InputElementRegExpOrTemplateTail lexical goal.
features: [generators]
---*/

var complete = false;
var iter, iterResult, str;
function* g() {
  str = `1${ yield }3${ 4 }5`;
  complete = true;
}

iter = g();

assert.sameValue(complete, false, 'generator initially paused');
assert.sameValue(str, undefined, 'first statement not executed');

iterResult = iter.next();

assert.sameValue(complete, false, 'generator paused following expression');
assert.sameValue(str, undefined, 'first statement not executed');

assert.sameValue(iterResult.done, false, 'iteration not complete');
assert.sameValue(iterResult.value, undefined, 'first iterated value');

iterResult = iter.next(2);

assert.sameValue(str, '12345', 'YieldExpression value');
assert.sameValue(complete, true, 'generator correctly re-started');
assert.sameValue(iterResult.done, true, 'iteration complete');
assert.sameValue(iterResult.value, undefined, 'second iterated value');
