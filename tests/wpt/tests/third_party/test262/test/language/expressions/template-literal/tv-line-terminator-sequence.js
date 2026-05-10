// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of line terminator sequences
info: |
    The TV of TemplateCharacter :: LineTerminatorSequence is the TRV of
    LineTerminatorSequence.
    The TRV of LineTerminatorSequence :: <LF> is the code unit value 0x000A.
    The TRV of LineTerminatorSequence :: <CR> is the code unit value 0x000A.
    The TRV of LineTerminatorSequence :: <LS> is the code unit value 0x2028.
    The TRV of LineTerminatorSequence :: <PS> is the code unit value 0x2029.
    The TRV of LineTerminatorSequence :: <CR><LF> is the sequence consisting of
    the code unit value 0x000A.
---*/


var calls;

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], '\n\n\n', 'Line Feed and Carriage Return');
  assert.sameValue(s.raw[0], '\n\n\n', 'Line Feed and Carriage Return');
})`

`;
assert.sameValue(calls, 1);

calls = 0;
(function(cs) {
  calls++;
  assert.sameValue(cs[0], '\u2028', 'Line Separator');
  assert.sameValue(cs.raw[0], '\u2028', 'Line Separator');
})` `
assert.sameValue(calls, 1);

calls = 0;
(function(cs) {
  calls++;
  assert.sameValue(cs[0], '\u2029', 'Paragraph Separator');
  assert.sameValue(cs.raw[0], '\u2029', 'Paragraph Separator');
})` `
assert.sameValue(calls, 1);
