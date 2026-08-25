// Copyright (C) 2018 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-literals-string-literals
description: >
  Line terminators may occur within string literals as part of a |LineContinuation|
  to produce the empty code points sequence.
info: |
  11.8.4 String Literals

  StringLiteral ::
    `"` DoubleStringCharacters? `"`
    `'` SingleStringCharacters? `'`

  SingleStringCharacters ::
    SingleStringCharacter SingleStringCharacters?

  SingleStringCharacter ::
    SourceCharacter but not one of `'` or `\` or LineTerminator
    <LS>
    <PS>
    `\` EscapeSequence
    LineContinuation

  LineContinuation ::
    `\` LineTerminatorSequence

  11.3 Line Terminators

  LineTerminatorSequence ::
    <LF>
    <CR> [lookahead != <LF>]
    <LS>
    <PS>
    <CR> <LF>

  11.8.4.2 Static Semantics: SV

  The SV of SingleStringCharacter :: LineContinuation is the empty code unit sequence.
---*/

// LineTerminatorSequence :: <LF>
assert.sameValue('\
', '');

// LineTerminatorSequence :: <CR> [lookahead ≠ <LF>]
assert.sameValue('\', '');

// LineTerminatorSequence :: <LS>
// <LS> is U+2028 LINE SEPARATOR; UTF8(0x2028) = 0xE2 0x80 0xA8
assert.sameValue('\ ', '');

// LineTerminatorSequence :: <PS>
// <PS> is U+2029 PARAGRAPH SEPARATOR; UTF8(0x2029) = 0xE2 0x80 0xA9
assert.sameValue('\ ', '');

// LineTerminatorSequence :: <CR> <LF>
assert.sameValue('\
', '');
