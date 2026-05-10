// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Extensions to NonemptyClassRangesNoDash production
info: |
    The production
    NonemptyClassRangesNoDash::ClassAtomNoDash-ClassAtomClassRanges evaluates
    as follows:

    1. Evaluate ClassAtomNoDash to obtain a CharSet A.
    2. Evaluate ClassAtom to obtain a CharSet B.
    3. Evaluate ClassRanges to obtain a CharSet C.
    4. Call CharacterRangeOrUnion(A, B) and let D be the resulting CharSet.
    5. Return the union of CharSets D and C.

    B.1.4.1.1 Runtime Semantics: CharacterRangeOrUnion Abstract Operation

    1. If Unicode is false, then
       a. If A does not contain exactly one character or B does not contain
          exactly one character, then
          i. Let C be the CharSet containing the single character - U+002D
             (HYPHEN-MINUS).
          ii. Return the union of CharSets A, B and C.
    2. Return CharacterRange(A, B).
---*/

var match;

match = /[\d-a]+/.exec(':a0123456789-:');
assert.sameValue(match[0], 'a0123456789-');

match = /[\d-az]+/.exec(':a0123456789z-:');
assert.sameValue(match[0], 'a0123456789z-');

match = /[%-\d]+/.exec('&%0123456789-&');
assert.sameValue(match[0], '%0123456789-');

match = /[%-\dz]+/.exec('&%0123456789z-&');
assert.sameValue(match[0], '%0123456789z-');

match = /[\s-\d]+/.exec('& \t0123456789-&');
assert.sameValue(match[0], ' \t0123456789-');

match = /[\s-\dz]+/.exec('& \t0123456789z-&');
assert.sameValue(match[0], ' \t0123456789z-');
