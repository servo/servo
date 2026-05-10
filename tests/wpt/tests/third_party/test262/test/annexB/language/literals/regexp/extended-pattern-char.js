// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Extended Pattern Characters (as distinct from Pattern Characters)
info: |
    ExtendedPatternCharacter ::
        SourceCharacterbut not one of ^$.*+?()[|

    The production ExtendedAtom::ExtendedPatternCharacter evaluates as follows:

    1. Let ch be the character represented by ExtendedPatternCharacter.
    2. Let A be a one-element CharSet containing the character ch.
    3. Call CharacterSetMatcher(A, false) and return its Matcher result.
---*/

var match;

match = /]/.exec(' ]{}');
assert.sameValue(match[0], ']');

match = /{/.exec(' ]{}');
assert.sameValue(match[0], '{');

match = /}/.exec(' ]{}');
assert.sameValue(match[0], '}');

match = /x{o}x/.exec('x{o}x');
assert.sameValue(match[0], 'x{o}x');
