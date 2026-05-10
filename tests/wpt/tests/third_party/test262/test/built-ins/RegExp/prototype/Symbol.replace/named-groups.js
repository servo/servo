// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-getsubstitution
description: >
  RegExp.prototype[Symbol.replace] works with named capture references as expected.
  (string replacement)
info: |
  GetSubstitution ( matched, str, position, captures, namedCaptures, replacement )

  Table: Replacement Text Symbol Substitutions

  Unicode Characters: $<
  Replacement text:
    1. If namedCaptures is undefined, the replacement text is the literal string $<.
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

assert.sameValue(/b/u[Symbol.replace]("abc", "$&$<food"), "ab$<foodc");
assert.sameValue(/./g[Symbol.replace]("ab", "c$<foo>d"), "c$<foo>dc$<foo>d");
assert.sameValue(/(b)./[Symbol.replace]("abc", "$<foo>$1"), "a$<foo>b");

assert.sameValue(/(?<foo>.)(?<bar>.)/[Symbol.replace]("abc", "$<bar>$<foo>"), "bac");
assert.sameValue(/(?<foo>.)(?<bar>.)/gu[Symbol.replace]("abc", "$2$<foo>$1"), "baac");
assert.sameValue(/(?<foo>b)/u[Symbol.replace]("abc", "c$<bar>d"), "acdc");
assert.sameValue(/(?<foo>.)/g[Symbol.replace]("abc", "$<$1>"), "");
assert.sameValue(/(?<foo>b)/[Symbol.replace]("abc", "$<>"), "ac");
assert.sameValue(/(?<foo>.)(?<bar>.)/g[Symbol.replace]("abc", "$2$1"), "bac");
assert.sameValue(/(?<foo>b)/u[Symbol.replace]("abc", "$<foo"), "a$<fooc");
assert.sameValue(/(?<foo>.)/gu[Symbol.replace]("abc", "$<bar>"), "");
assert.sameValue(/(?<foo>b)/[Symbol.replace]("abc", "$$<foo>$&"), "a$<foo>bc");

assert.sameValue(/(?<𝒜>b)/u[Symbol.replace]("abc", "d$<𝒜>$`"), "adbac");
assert.sameValue(/(?<$𐒤>b)/gu[Symbol.replace]("abc", "$'$<$𐒤>d"), "acbdc");
