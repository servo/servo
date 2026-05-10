// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  pending
esid: pending
---*/

testJSONSyntaxError("{}...");
testJSONSyntaxError('{"foo": truBBBB}');
testJSONSyntaxError('{foo: truBBBB}');
testJSONSyntaxError('{"foo": undefined}');
testJSONSyntaxError('{"foo": ]');
testJSONSyntaxError('{"foo');
