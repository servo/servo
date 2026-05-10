// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of single characters
info: |
    The TV of TemplateCharacters :: TemplateCharacter is the TV of
    TemplateCharacter.
    The TV of TemplateCharacter :: SourceCharacter but not one of ` or \ or $
    or LineTerminator is the UTF16Encoding (10.1.1) of the code point value of
    SourceCharacter.
    The TV of TemplateCharacter :: $ is the code unit value 0x0024.

    The TRV of TemplateCharacters :: TemplateCharacter is the TRV of
    TemplateCharacter.
    The TRV of TemplateCharacter :: SourceCharacter but not one of ` or \ or $
    or LineTerminator is the UTF16Encoding (10.1.1) of the code point value of
    SourceCharacter.
    The TRV of TemplateCharacter :: $ is the code unit value 0x0024.
---*/

var calls;

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], 'a', '`a` character TV');
  assert.sameValue(s.raw[0], 'a', '`a` character TRV');
})`a`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], '$', '`$` character TV');
  assert.sameValue(s.raw[0], '$', '`$` character TRV');
})`$`;
assert.sameValue(calls, 1);
