// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Literal astral symbols within a pattern
es6id: 21.2.2.8.2
info: |
    21.2.2.2 Pattern

    The production Pattern :: Disjunction evaluates as follows:

        1. Evaluate Disjunction to obtain a Matcher m.
        2. Return an internal closure that takes two arguments, a String str
           and an integer index, and performs the following steps:
           1. If Unicode is true, let Input be a List consisting of the
              sequence of code points of str interpreted as a UTF-16 encoded
              (6.1.4) Unicode string. Otherwise, let Input be a List consisting
              of the sequence of code units that are the elements of str. Input
              will be used throughout the algorithms in 21.2.2. Each element of
              Input is considered to be a character.
---*/

assert(/𝌆{2}/u.test('𝌆𝌆'), 'quantifier application');

assert(/^[𝌆]$/u.test('𝌆'), 'as a ClassAtom');

var rangeRe = /[💩-💫]/u;
assert.sameValue(
  rangeRe.test('\ud83d\udca8'),
  false,
  'ClassAtom as lower range boundary, input below (U+1F4A8)'
);
assert.sameValue(
  rangeRe.test('\ud83d\udca9'),
  true,
  'ClassAtom as lower range boundary, input match (U+1F4A9)'
);
assert.sameValue(
  rangeRe.test('\ud83d\udcaa'),
  true,
  'ClassAtom as upper- and lower-range boundary, input within (U+1F4AA)'
);
assert.sameValue(
  rangeRe.test('\ud83d\udcab'),
  true,
  'ClassAtom as upper range boundary, input match (U+1F4AB)'
);
assert.sameValue(
  rangeRe.test('\ud83d\udcac'),
  false,
  'ClassAtom as upper range boundary, input above (U+1F4AC)'
);

assert(/[^𝌆]/u.test('\ud834'), 'Negated character classes (LeadSurrogate)');
assert(/[^𝌆]/u.test('\udf06'), 'Negated character classes (TrailSurrogate)');
