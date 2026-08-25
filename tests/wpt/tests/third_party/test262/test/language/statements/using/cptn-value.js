// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Returns an empty completion
info: |
  UsingDeclaration : using BindingList ;

  1. Perform ? BindingEvaluation of BindingList with argument sync-dispose.
  2. Return empty.

features: [explicit-resource-management]
---*/

assert.sameValue(
  eval('{using test262id1 = null;}'), undefined, 'Single declaration'
);
assert.sameValue(
  eval('{using test262id2 = null, test262id3 = null;}'),
  undefined,
  'Multiple declarations'
);

assert.sameValue(eval('4; {using test262id5 = null;}'), 4);
assert.sameValue(eval('6; {using test262id7 = null, test262id8 = null;}'), 6);
