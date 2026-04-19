// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Basic matching cases with non-unicode matches.
includes: [compareArray.js, propertyHelper.js, deepEqual.js]
esid: sec-regexpbuiltinexec
features: [regexp-named-groups, regexp-match-indices]
info: |
  Runtime Semantics: RegExpBuiltinExec ( R, S )
    ...
    4. Let _lastIndex_ be ? ToLength(? Get(_R_, `"lastIndex")).
    ...
    16. If _fullUnicode_ is *true*, set _e_ to ! GetStringIndex(_S_, _Input_, _e_).
    ...
    26. Let _match_ be the Match { [[StartIndex]]: _lastIndex_, [[EndIndex]]: _e_ }.
    27. Let _indices_ be a new empty List.
    ...
    29. Add _match_ as the last element of _indices_.
    ...
    35. For each integer _i_ such that _i_ > 0 and _i_ <= _n_, in ascending order, do
      ...
      f. Else,
        i. Let _captureStart_ be _captureI_'s _startIndex_.
        ii. Let _captureEnd_ be _captureI_'s _endIndex_.
        iii. If _fullUnicode_ is *true*, then
          1. Set _captureStart_ to ! GetStringIndex(_S_, _Input_, _captureStart_).
          1. Set _captureEnd_ to ! GetStringIndex(_S_, _Input_, _captureEnd_).
        iv. Let _capture_ be the Match  { [[StartIndex]]: _captureStart_, [[EndIndex]]: _captureEnd_ }.
        v. Append _capture_ to _indices_.
        ...
    36. If _hasIndices_ is *true*, then
      a. Let _indicesArray_ be MakeIndicesArray(_S_, _indices_, _groupNames_, _hasGroups_).
      b. Perform ! CreateDataProperty(_A_, `"indices"`, _indicesArray_).

  GetStringIndex ( S, Input, e )
    ...
    4. Let _eUTF_ be the smallest index into _S_ that corresponds to the character at element _e_ of _Input_. If _e_ is greater than or equal to the number of elements in _Input_, then _eUTF_ is the number of code units in _S_.
    5. Return _eUTF_.
---*/

assert.deepEqual([[1, 2], [1, 2]], "bab".match(/(a)/du).indices);
assert.deepEqual([[0, 3], [1, 2]], "bab".match(/.(a)./du).indices);
assert.deepEqual([[0, 3], [1, 2], [2, 3]], "bab".match(/.(a)(.)/du).indices);
assert.deepEqual([[0, 3], [1, 3]], "bab".match(/.(\w\w)/du).indices);
assert.deepEqual([[0, 3], [0, 3]], "bab".match(/(\w\w\w)/du).indices);
assert.deepEqual([[0, 3], [0, 2], [2, 3]], "bab".match(/(\w\w)(\w)/du).indices);
assert.deepEqual([[0, 2], [0, 2], undefined], "bab".match(/(\w\w)(\W)?/du).indices);

let groups = /(?<a>.)(?<b>.)(?<c>.)\k<c>\k<b>\k<a>/du.exec("abccba").indices.groups;
assert.compareArray([0, 1], groups.a);
assert.compareArray([1, 2], groups.b);
assert.compareArray([2, 3], groups.c);
verifyProperty(groups, "a", {
    enumerable: true,
    writable: true,
    configurable: true
});
verifyProperty(groups, "b", {
    enumerable: true,
    writable: true,
    configurable: true
});
verifyProperty(groups, "c", {
    enumerable: true,
    writable: true,
    configurable: true
});

// "𝐁" is U+1d401 MATHEMATICAL BOLD CAPITAL B
// - Also representable as the code point "\u{1d401}"
// - Also representable as the surrogate pair "\uD835\uDC01"

// Verify assumptions:
assert.sameValue("𝐁".length, 2, 'The length of "𝐁" is 2');
assert.sameValue("\u{1d401}".length, 2, 'The length of "\\u{1d401}" is 2');
assert.sameValue("\uD835\uDC01".length, 2, 'The length of "\\uD835\\uDC01" is 2');
assert.sameValue(2, "𝐁".match(/./u)[0].length, 'The length of a single code point match against "𝐁" is 2 (with /du flag)');
assert.sameValue(2, "\u{1d401}".match(/./u)[0].length, 'The length of a single code point match against "\\u{1d401}" is 2 (with /du flag)');
assert.sameValue(2, "\uD835\uDC01".match(/./u)[0].length, 'The length of a single code point match against "\\ud835\\udc01" is 2 (with /du flag)');

assert.compareArray([0, 2], "𝐁".match(/./du).indices[0], 'Indices for unicode match against "𝐁" (with /du flag)');
assert.compareArray([0, 2], "\u{1d401}".match(/./du).indices[0], 'Indices for unicode match against \\u{1d401} (with /du flag)');
assert.compareArray([0, 2], "\uD835\uDC01".match(/./du).indices[0], 'Indices for unicode match against \\ud835\\udc01 (with /du flag)');
assert.compareArray([0, 2], "𝐁".match(/(?<a>.)/du).indices.groups.a, 'Indices for unicode match against 𝐁 in groups.a (with /du flag)');
assert.compareArray([0, 2], "\u{1d401}".match(/(?<a>.)/du).indices.groups.a, 'Indices for unicode match against \\u{1d401} in groups.a (with /du flag)');
assert.compareArray([0, 2], "\uD835\uDC01".match(/(?<a>.)/du).indices.groups.a, 'Indices for unicode match against \\ud835\\udc01 in groups.a (with /du flag)');
