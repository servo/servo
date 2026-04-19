// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of character escape sequences
info: |
    The TV of TemplateCharacter :: \ EscapeSequence is the SV of
    EscapeSequence.
    The TRV of TemplateCharacter :: \ EscapeSequence is the sequence consisting
    of the code unit value 0x005C followed by the code units of TRV of
    EscapeSequence.
    The TRV of CharacterEscapeSequence :: SingleEscapeCharacter is the TRV of
    the SingleEscapeCharacter.
    The TRV of CharacterEscapeSequence :: NonEscapeCharacter is the SV of the
    NonEscapeCharacter.
---*/
var calls;

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "'", "TV of NonEscapeCharacter");
  assert.sameValue(s.raw[0], "\u005C\u0027", "TRV of NonEscapeCharacter");
})`\'`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "\"", "TV of SingleEscapeCharacter (double quote)");
  assert.sameValue(
    s.raw[0], "\u005C\u0022", "TRV of SingleEscapeCharacter (double quote)"
  );
})`\"`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "\\", "TV of SingleEscapeCharacter (backslash)");
  assert.sameValue(
    s.raw[0], "\u005C\u005C", "TRV of SingleEscapeCharacter (backslash)"
  );
})`\\`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "\b", "TV of SingleEscapeCharacter (backspace)");
  assert.sameValue(
    s.raw[0], "\u005Cb", "TRV of SingleEscapeCharacter (backspace)"
  );
})`\b`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "\f", "TV of SingleEscapeCharacter (form feed)");
  assert.sameValue(
    s.raw[0], "\u005Cf", "TRV of SingleEscapeCharacter (form feed)"
  );
})`\f`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "\n", "TV of SingleEscapeCharacter (new line)");
  assert.sameValue(
    s.raw[0], "\u005Cn", "TRV of SingleEscapeCharacter (new line)"
  );
})`\n`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(
    s[0], "\r", "TV of SingleEscapeCharacter (carriage return)"
  );
  assert.sameValue(
    s.raw[0], "\u005Cr", "TRV of SingleEscapeCharacter (carriage return)"
  );
})`\r`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "	", "TV of SingleEscapeCharacter (tab)");
  assert.sameValue(s.raw[0], "\u005Ct", "TRV of SingleEscapeCharacter (tab)");
})`\t`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(
    s[0], "\v", "TV of SingleEscapeCharacter (line tabulation)"
  );
  assert.sameValue(
    s.raw[0], "\u005Cv", "TRV of SingleEscapeCharacter (line tabulation)"
  );
})`\v`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[0], "`", "TV of SingleEscapeCharacter (backtick)");
  assert.sameValue(
    s.raw[0], "\u005C`", "TRV of SingleEscapeCharacter (backtick)"
  );
})`\``;
assert.sameValue(calls, 1);
