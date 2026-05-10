// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: >
    Behavior when pattern is a string describing a valid pattern (unicode)
info: |
    [...]
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    6. If F contains "u", let BMP be false; else let BMP be true.
    7. If BMP is true, then
       a. Parse P using the grammars in 21.2.1 and interpreting each of its
          16-bit elements as a Unicode BMP code point. UTF-16 decoding is not
          applied to the elements. The goal symbol for the parse is Pattern.
          Throw a SyntaxError exception if P did not conform to the grammar, if
          any elements of P were not matched by the parse, or if any Early
          Error conditions exist.
       b. Let patternCharacters be a List whose elements are the code unit
          elements of P.
    [...]
---*/

var subject = /original value/ig;

subject.compile('[\ud834\udf06]', 'u');

assert.sameValue(
  subject.source,
  new RegExp('[\ud834\udf06]', 'u').source,
  '[[OriginalSource]] internal slot'
);
assert.sameValue(
  subject.test('original value'),
  false,
  '[[RegExpMatcher]] internal slot (source)'
);
assert.sameValue(
  subject.test('\ud834'), false, '[[RegExpMatcher]] internal slot (flags #1)'
);
assert.sameValue(
  subject.test('\udf06'), false, '[[RegExpMatcher]] internal slot (flags #2)'
);
assert.sameValue(
  subject.test('\ud834\udf06'),
  true,
  '[[RegExpMatcher]] internal slot (flags #3)'
);
