// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-strict-mode-of-ecmascript
description: >
  It is a SyntaxError if FormalParameters contains eval in strict mode.
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
features: [async-iteration]
---*/

$DONOTEVALUATE();

(async function*(eval) { });
