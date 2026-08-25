// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Extensions to ClassEscape
info: |
    ClassEscape[U] ::
        b
        [+U] -
        [~U] c ClassControlLetter
        CharacterClassEscape
        CharacterEscape[?U]

    ClassControlLetter ::
        DecimalDigit
        _

    The production ClassEscape :: c ClassControlLetter evaluates as follows:

    1. Let ch be the character matched by ClassControlLetter.
    2. Let i be ch's character value.
    3. Let j be the remainder of dividing i by 32.
    4. Let d be the character whose character value is j.
    5. Return the CharSet containing the single character d.
---*/

var match;

match = /\c0/.exec('\x0f\x10\x11');
assert.sameValue(match, null, '\\c0 outside of CharacterClass');

match = /[\c0]/.exec('\x0f\x10\x11');
assert.sameValue(match[0], '\x10', '\\c0 within CharacterClass');

match = /[\c00]+/.exec('\x0f0\x10\x11');
assert.sameValue(match[0], '0\x10', '\\c00 within CharacterClass');

match = /\c1/.exec('\x10\x11\x12');
assert.sameValue(match, null, '\\c1 outside of CharacterClass');

match = /[\c1]/.exec('\x10\x11\x12');
assert.sameValue(match[0], '\x11', '\\c1 within CharacterClass');

match = /[\c10]+/.exec('\x100\x11\x12');
assert.sameValue(match[0], '0\x11', '\\c10 within CharacterClass');

match = /\c8/.exec('\x17\x18\x19');
assert.sameValue(match, null, '\\c8 outside of CharacterClass');

match = /[\c8]/.exec('\x17\x18\x19');
assert.sameValue(match[0], '\x18', '\\c8 within CharacterClass');

match = /[\c80]+/.exec('\x170\x18\x19');
assert.sameValue(match[0], '0\x18', '\\c80 within CharacterClass');

match = /\c9/.exec('\x18\x19\x1a');
assert.sameValue(match, null, '\\c9 outside of CharacterClass');

match = /[\c9]/.exec('\x18\x19\x1a');
assert.sameValue(match[0], '\x19', '\\c9 within CharacterClass');

match = /[\c90]+/.exec('\x180\x19\x1a');
assert.sameValue(match[0], '0\x19', '\\c90 within CharacterClass');

match = /\c_/.exec('\x1e\x1f\x20');
assert.sameValue(match, null, '\\c_ outside of CharacterClass');

match = /[\c_]/.exec('\x1e\x1f\x20');
assert.sameValue(match[0], '\x1f', '\\c_ within CharacterClass');
