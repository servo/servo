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
includes: [compareArray.js]
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

const t = (function() { return this; })();
let calls;

function getFn(val) {
  return function replaceValueFn(...args) {
    calls.push([this, ...args]);
    return val;
  };
}

const samples = [
  [ '(a)', 'aaa abc', 'z', 'zzz zbc' ],
  [ '(a)', 'aaa abc', '$1', '$1$1$1 $1bc' ],
  [ '(a)', 'aaa abc', '$$', '$$$$$$ $$bc' ],
  [ '(a)', 'aaa abc', '$&', '$&$&$& $&bc' ],
  [ '(a)', 'aaa abc', '$\'', '$\'$\'$\' $\'bc' ],
  [ '(a)', 'aaa abc', '$`', '$`$`$` $`bc' ],
];

let count = 0;
for (const [ reStr, thisValue, replaceValue, expected ] of samples) {
  const searchValue = new RE(reStr, 'g');
  const replaceFn = getFn(replaceValue);

  // Observes the toString
  const obj = new String(thisValue);

  called = 0;
  calls = [];

  const actual = obj.replaceAll(searchValue, replaceFn);

  const message = `sample ${count}: '${thisValue}'.replaceAll(/${reStr}/g, () => '${replaceValue}')`;

  assert.sameValue(called, 1, `called -- ${message}`);
  assert.sameValue(actual, expected, `actual -- ${message}`);

  assert.sameValue(calls.length, 4, `calls.length -- ${message}`);
  assert.compareArray(calls[0], [t, 'a', 'a', 0, thisValue]);
  assert.compareArray(calls[1], [t, 'a', 'a', 1, thisValue]);
  assert.compareArray(calls[2], [t, 'a', 'a', 2, thisValue]);
  assert.compareArray(calls[3], [t, 'a', 'a', 4, thisValue]);

  count += 1;
}

const samplesSticky = [
  [ '(a)', 'aaa abc', 'z', 'zzz abc' ],
  [ '(a)', 'aaa abc', '$1', '$1$1$1 abc' ],
  [ '(a)', 'aaa abc', '$$', '$$$$$$ abc' ],
  [ '(a)', 'aaa abc', '$&', '$&$&$& abc' ],
  [ '(a)', 'aaa abc', '$\'', '$\'$\'$\' abc' ],
  [ '(a)', 'aaa abc', '$`', '$`$`$` abc' ],
];

count = 0;
for (const [ reStr, thisValue, replaceValue, expected ] of samplesSticky) {
  const searchValue = new RE(reStr, 'gy');
  const replaceFn = getFn(replaceValue);

  // Observes the toString
  const obj = new String(thisValue);

  called = 0;
  calls = [];

  const actual = obj.replaceAll(searchValue, replaceFn);

  const message = `sample ${count}: '${thisValue}'.replaceAll(/${reStr}/gy, () => '${replaceValue}')`;

  assert.sameValue(called, 1, `called -- ${message}`);
  assert.sameValue(actual, expected, `actual -- ${message}`);

  assert.sameValue(calls.length, 3, `calls.length -- ${message}`);
  assert.compareArray(calls[0], [t, 'a', 'a', 0, thisValue]);
  assert.compareArray(calls[1], [t, 'a', 'a', 1, thisValue]);
  assert.compareArray(calls[2], [t, 'a', 'a', 2, thisValue]);

  count += 1;
}
