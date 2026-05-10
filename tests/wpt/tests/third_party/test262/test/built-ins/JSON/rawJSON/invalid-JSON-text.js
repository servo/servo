// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.rawjson
description: >
  Inputs not convertible to string, or convertible to invalid JSON string
info: |
  JSON.rawJSON ( text )

  1. Let jsonString be ? ToString(text).
  ...
  3. Parse StringToCodePoints(jsonString) as a JSON text as specified in
     ECMA-404. Throw a SyntaxError exception if it is not a valid JSON text as
     defined in that specification, or if its outermost value is an object or
     array as defined in that specification.

features: [json-parse-with-source]
---*/

assert.throws(TypeError, () => {
  JSON.rawJSON(Symbol('123'));
});

assert.throws(SyntaxError, () => {
  JSON.rawJSON(undefined);
});

assert.throws(SyntaxError, () => {
  JSON.rawJSON({});
});

assert.throws(SyntaxError, () => {
  JSON.rawJSON([]);
});
