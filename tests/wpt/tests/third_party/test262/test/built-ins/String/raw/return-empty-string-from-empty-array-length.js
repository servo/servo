// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  Returns empty string if template.raw an empty Array
info: |
  21.1.2.4 String.raw ( template , ...substitutions )

  ...
  7. Let literalSegments be ToLength(Get(raw, "length")).
  8. ReturnIfAbrupt(literalSegments).
  9. If literalSegments ≤ 0, return the empty string.
  ...
---*/

var result = String.raw({
  raw: []
});

assert.sameValue(result, '');
