// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
es6id: 13.3.1.4
description: Returns an empty completion
info: |
  LexicalDeclaration : LetOrConst BindingList ;

  1. Let next be the result of evaluating BindingList.
  2. ReturnIfAbrupt(next).
  3. Return NormalCompletion(empty).
---*/

assert.sameValue(
  eval('let test262id1;'), undefined, 'Single declaration without initializer'
);
assert.sameValue(
  eval('let test262id2 = 2;'),
  undefined,
  'Single declaration bearing initializer'
);
assert.sameValue(
  eval('let test262id3 = 3, test262id4;'),
  undefined,
  'Multiple declarations, final without initializer'
);
assert.sameValue(
  eval('let test262id5, test262id6 = 6;'),
  undefined,
  'Multiple declarations, final bearing initializer'
);

assert.sameValue(eval('7; let test262id8;'), 7);
assert.sameValue(eval('9; let test262id10 = 10;'), 9);
assert.sameValue(eval('11; let test262id12 = 12, test262id13;'), 11);
assert.sameValue(eval('14; let test262id15, test262id16 = 16;'), 14);
