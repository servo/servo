// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  String coercion of "groups" object values returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    j. Let namedCaptures be ? Get(result, "groups").
    k. If functionalReplace is true, then
      [...]
    l. Else,
      [...]
      ii. Let replacement be ? GetSubstitution(matched, S, position, captures, namedCaptures, replaceValue).

  Runtime Semantics: GetSubstitution ( matched, str, position, captures, namedCaptures, replacement )

  [...]
  11. Let result be the String value derived from replacement by copying code unit elements
  from replacement to result while performing replacements as specified in Table 54.
  These $ replacements are done left-to-right, and, once such a replacement is performed,
  the new replacement text is not subject to further replacements.
  12. Return result.

  Table 54: Replacement Text Symbol Substitutions

  $<

  1. If namedCaptures is undefined, the replacement text is the String "$<".
  2. Else,
    a. Assert: Type(namedCaptures) is Object.
    b. Scan until the next > U+003E (GREATER-THAN SIGN).
    c. If none is found, the replacement text is the String "$<".
    d. Else,
      i. Let groupName be the enclosed substring.
      ii. Let capture be ? Get(namedCaptures, groupName).
      iii. If capture is undefined, replace the text through > with the empty String.
      iv. Otherwise, replace the text through > with ? ToString(capture).
features: [Symbol.replace, regexp-named-groups]
---*/

var r = /./;
var coercibleValue = {
  length: 1,
  0: 'a',
  index: 0,
  groups: {
    foo: {
      toString: function() {
        return 'toString value';
      },
      valueOf: function() {
        throw new Test262Error('This method should not be invoked.');
      },
    },
    bar: null,
  },
};

r.exec = function() {
  return coercibleValue;
};

assert.sameValue(r[Symbol.replace]('ab', '[$<foo>]'), '[toString value]b');
assert.sameValue(r[Symbol.replace]('ab', '[$<bar>]'), '[null]b');
