// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Returns false if search start is less than 0.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  ...
  9. Let len be the number of elements in S.
  10. If endPosition is undefined, let pos be len, else let pos be
  ToInteger(endPosition).
  11. ReturnIfAbrupt(pos).
  12. Let end be min(max(pos, 0), len).
  13. Let searchLength be the number of elements in searchStr.
  14. Let start be end - searchLength.
  15. If start is less than 0, return false.
  ...

  Note: (min(max(pos, 0), len) - searchString.length) < 0;
features: [String.prototype.endsWith]
---*/

assert.sameValue(
  'web'.endsWith('w', 0), false,
  '"web".endsWith("w", 0) returns false'
);

assert.sameValue(
  'Bob'.endsWith('  Bob'), false,
  '"Bob".endsWith("  Bob") returns false'
);
