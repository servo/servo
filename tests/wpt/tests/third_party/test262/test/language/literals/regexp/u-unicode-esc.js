// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Unicode escape interpreted as the Mathematical Value of HexDigits
es6id: 21.2.2.10
info: |
    21.2.2.10 CharacterEscape

    The production RegExpUnicodeEscapeSequence :: u{ HexDigits } evaluates as
    follows:

        1. Return the character whose code is the MV of HexDigits.
---*/

assert(/\u{0}/u.test('\u0000'), 'Minimum value (U+0000)');
assert(/\u{1}/u.test('\u0001'), 'U+0001');
assert.sameValue(/\u{1}/u.test('u'), false);
assert(/\u{3f}/u.test('?'));
assert(/\u{000000003f}/u.test('?'), 'Leading zeros');
assert(/\u{3F}/u.test('?'), 'Case insensitivity');
assert(/\u{10ffff}/u.test('\udbff\udfff'), 'Maxiumum value (U+10ffff)');
