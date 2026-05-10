// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
description: Legacy Octal Escape Sequence
info: |
    CharacterEscape[U] ::
        ControlEscape
        c ControlLetter
        0 [lookahead ∉ DecimalDigit]
        HexEscapeSequence
        RegExpUnicodeEscapeSequence[?U]
        [~U] LegacyOctalEscapeSequence
        IdentityEscape[?U]

    LegacyOctalEscapeSequence ::
        OctalDigit [lookahead ∉ OctalDigit]
        ZeroToThree OctalDigit [lookahead ∉ OctalDigit]
        FourToSeven OctalDigit
        ZeroToThree OctalDigit OctalDigit

    The production CharacterEscape :: LegacyOctalEscapeSequence evaluates by
    evaluating the SV of the LegacyOctalEscapeSequence and returning its
    character result.
---*/

assert.sameValue(/\1/.exec('\x01')[0], '\x01', '\\1');
assert.sameValue(/\2/.exec('\x02')[0], '\x02', '\\2');
assert.sameValue(/\3/.exec('\x03')[0], '\x03', '\\3');
assert.sameValue(/\4/.exec('\x04')[0], '\x04', '\\4');
assert.sameValue(/\5/.exec('\x05')[0], '\x05', '\\5');
assert.sameValue(/\6/.exec('\x06')[0], '\x06', '\\6');
assert.sameValue(/\7/.exec('\x07')[0], '\x07', '\\7');

assert.sameValue(/\00/.exec('\x00')[0], '\x00', '\\00');
assert.sameValue(/\07/.exec('\x07')[0], '\x07', '\\07');

assert.sameValue(/\30/.exec('\x18')[0], '\x18', '\\30');
assert.sameValue(/\37/.exec('\x1f')[0], '\x1f', '\\37');

assert.sameValue(/\40/.exec('\x20')[0], '\x20', '\\40');
assert.sameValue(/\47/.exec('\x27')[0], '\x27', '\\47');

assert.sameValue(/\70/.exec('\x38')[0], '\x38', '\\70');
assert.sameValue(/\77/.exec('\x3f')[0], '\x3f', '\\77');

// Sequence is bounded according to the String Value
assert.sameValue(/\400/.exec('\x200')[0], '\x200', '\\400');
assert.sameValue(/\470/.exec('\x270')[0], '\x270', '\\470');
assert.sameValue(/\700/.exec('\x380')[0], '\x380', '\\700');
assert.sameValue(/\770/.exec('\x3f0')[0], '\x3f0', '\\770');

assert.sameValue(/\000/.exec('\x00')[0], '\x00', '\\000');
assert.sameValue(/\007/.exec('\x07')[0], '\x07', '\\007');
assert.sameValue(/\070/.exec('\x38')[0], '\x38', '\\070');

assert.sameValue(/\300/.exec('\xc0')[0], '\xc0', '\\300');
assert.sameValue(/\307/.exec('\xc7')[0], '\xc7', '\\307');
assert.sameValue(/\370/.exec('\xf8')[0], '\xf8', '\\370');
assert.sameValue(/\377/.exec('\xff')[0], '\xff', '\\377');

// Sequence is 3 characters max, including leading zeros
assert.sameValue(/\0111/.exec('\x091')[0], '\x091', '\\0111');
assert.sameValue(/\0022/.exec('\x022')[0], '\x022', '\\0022');
assert.sameValue(/\0003/.exec('\x003')[0], '\x003', '\\0003');

var match = /(.)\1/.exec('a\x01 aa');
assert.sameValue(
  match[0], 'aa', 'DecimalEscape takes precedence over LegacyOctalEscapeSequence'
);
