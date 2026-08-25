// Copyright (C) 2016 Tim Disney. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Invalid unicode escape sequence in tagged template literals are allowed
esid: sec-template-literal-lexical-components
---*/

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\01');
})`\01`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\1');
})`\1`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\8');
})`\8`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\9');
})`\9`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\xg');
})`\xg`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\xAg');
})`\xAg`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u0');
})`\u0`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u0g');
})`\u0g`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u00g');
})`\u00g`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u000g');
})`\u000g`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u{g');
})`\u{g`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u{0');
})`\u{0`;

(strs => {
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u{10FFFFF}');
})`\u{10FFFFF}`;

((strs, val) => {
  assert.sameValue(val, 'inner');
  assert.sameValue(strs[0], undefined, 'Cooked template value should be undefined for illegal escape sequences');
  assert.sameValue(strs.raw[0], '\\u{10FFFFF}');

  assert.sameValue(strs[1], 'right');
  assert.sameValue(strs.raw[1], 'right');
})`\u{10FFFFF}${'inner'}right`;
