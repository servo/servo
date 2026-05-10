// Copyright 2018 Leonardo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-characterclassescape
description: Detect non WhiteSpace using \S+
info: |
    The production CharacterClassEscape :: S evaluates by returning
    the set of all characters not included in the set returned by
    CharacterClassEscape :: s

    The Mongolian Vowel Separator (u180e) became a non whitespace character
    since Unicode 6.3.0
features: [u180e]
---*/

var str = String.fromCharCode(0x180E);
assert.sameValue(
  str.replace(/\S+/g, "test262"),
  "test262",
  "Non WhiteSpace character: \\u180E"
);
