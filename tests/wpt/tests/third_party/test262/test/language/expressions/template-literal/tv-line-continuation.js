// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of line continuations
info: |
    The TV of LineContinuation :: \ LineTerminatorSequence is the empty code
    unit sequence.
    The TRV of LineContinuation :: \ LineTerminatorSequence is the sequence
    consisting of the code unit value 0x005C followed by the code units of TRV
    of LineTerminatorSequence.
---*/

var calls;

calls = 0;
(function(cs) {
  calls++;
  assert.sameValue(cs[0], '', 'Line Feed and Carriage Return');
  assert.sameValue(
    cs.raw[0], '\u005C\n\u005C\n\u005C\n', 'Line Feed and Carriage Return'
  );
})`\
\
\`
assert.sameValue(calls, 1);

calls = 0;
(function(cs) {
  calls++;
  assert.sameValue(cs[0], '', 'Line Separator');
  assert.sameValue(cs.raw[0], '\\\u2028', 'Line Separator');
})`\ `
assert.sameValue(calls, 1);

calls = 0;
(function(cs) {
  calls++;
  assert.sameValue(cs[0], '', 'Paragraph Separater');
  assert.sameValue(cs.raw[0], '\\\u2029', 'Paragraph Separator');
})`\ `
assert.sameValue(calls, 1);
