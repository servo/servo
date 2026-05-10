// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  A RegExp searchValue's Symbol.replace can be called instead of the next steps of replaceAll
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    a. Let isRegExp be ? IsRegExp(searchString).
    b. If isRegExp is true, then
      i. Let flags be ? Get(searchValue, "flags").
      ii. Perform ? RequireObjectCoercible(flags).
      iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
    c. Let replacer be ? GetMethod(searchValue, @@replace).
    d. If replacer is not undefined, then
      i. Return ? Call(replacer, searchValue, « O, replaceValue »).
  3. Let string be ? ToString(O).
  4. Let searchString be ? ToString(searchValue).
  ...
features: [String.prototype.replaceAll, Symbol.replace, class]
---*/

let called = 0;

class RE extends RegExp {
  [Symbol.replace](...args) {
    const actual = super[Symbol.replace](...args);

    // Ordering is intentional to observe call from super
    called += 1;
    return actual;
  }

  toString() {
    throw 'Should not call toString on searchValue';
  }
}

const samples = [
  [ ['b', 'g'], 'abc abc abc', 'z', 'azc azc azc' ],
  [ ['b', 'gy'], 'abc abc abc', 'z', 'abc abc abc' ],
  [ ['b', 'giy'], 'abc abc abc', 'z', 'abc abc abc' ],
  [ [ '[A-Z]', 'g' ], 'No Uppercase!', '', 'o ppercase!' ],
  [ [ '[A-Z]', 'gy' ], 'No Uppercase?', '', 'o Uppercase?' ],
  [ [ '[A-Z]', 'gy' ], 'NO UPPERCASE!', '', ' UPPERCASE!' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '$2-$1', 'ca-bbcca-bbc' ],
  [ [ '(a(.))', 'g' ], 'abcabcabcabc', '$1$2$3', 'abb$3cabb$3cabb$3cabb$3c' ],
  [ [ '(((((((((((((a(.).).).).).).).).))))))', 'g' ], 'aabacadaeafagahaiajakalamano a azaya', '($10)-($12)-($1)', '(aabaca)-(aaba)-(aabacadaea)f(agahai)-(agah)-(agahaiajak)(alaman)-(alam)-(alamano a )azaya' ],
  [ [ 'b', 'g' ], 'abcba', '$\'', 'acbacaa' ],
  [ [ 'b', 'g' ], 'abcba', '$`', 'aacabca' ],
  [ [ '(?<named>b)', 'g' ], 'abcba', '($<named>)', 'a(b)c(b)a' ],
  [ [ '(?<named>b)', 'g' ], 'abcba', '($<named)', 'a($<named)c($<named)a' ],
  [ [ '(?<named>b)', 'g' ], 'abcba', '($<unnamed>)', 'a()c()a' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$)', '($)bc($)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($)', '($)bc($)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$$$)', '($$)bc($$)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$$)', '($$)bc($$)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$&)', '($&)bc($&)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$1)', '($1)bc($1)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$`)', '($`)bc($`)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($$\')', '($\')bc($\')bc' ],
  [ [ 'a(?<z>b)(ca)', 'g' ], 'abcabcabcabc', '($$<z>)', '($<z>)bc($<z>)bc' ],
  [ [ 'a(b)(ca)', 'g' ], 'abcabcabcabc', '($&)', '(abca)bc(abca)bc' ],
];

let count = 0;
for (const [ [ reStr, flags ], thisValue, replaceValue, expected ] of samples) {
  const searchValue = new RE(reStr, flags);

  called = 0;
  const actual = thisValue.replaceAll(searchValue, replaceValue);

  const message = `sample ${count}: '${thisValue}'.replaceAll(/${reStr}/${flags}, '${replaceValue}')`;

  assert.sameValue(called, 1, message);
  assert.sameValue(actual, expected, message);
  count += 1;
}
