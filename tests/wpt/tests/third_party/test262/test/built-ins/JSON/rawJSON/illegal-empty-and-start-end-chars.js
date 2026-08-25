// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.rawjson
description: Throw SyntaxError on empty string, or illegal start/end chars
info: |
  JSON.rawJSON ( text )

  1. Let jsonString be ? ToString(text).
  2. Throw a SyntaxError exception if jsonString is the empty String, or if
     either the first or last code unit of jsonString is any of 0x0009
     (CHARACTER TABULATION), 0x000A (LINE FEED), 0x000D (CARRIAGE RETURN), or
     0x0020 (SPACE).

features: [json-parse-with-source]
---*/

const ILLEGAL_END_CHARS = ['\n', '\t', '\r', ' '];
for (const char of ILLEGAL_END_CHARS) {
  assert.throws(SyntaxError, () => {
    JSON.rawJSON(`${char}123`);
  });
  assert.throws(SyntaxError, () => {
    JSON.rawJSON(`123${char}`);
  });
}

assert.throws(SyntaxError, () => {
  JSON.rawJSON('');
});
