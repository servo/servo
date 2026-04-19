// Copyright 2023 Mathias Bynens. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Mathias Bynens
description: >
  Some previously valid patterns with the `u` flag are now expected to
  throw an early SyntaxError with the `v` flag.
  https://github.com/tc39/proposal-regexp-v-flag#how-is-the-v-flag-different-from-the-u-flag
esid: sec-parsepattern
negative:
  phase: parse
  type: SyntaxError
features: [regexp-v-flag]
---*/

$DONOTEVALUATE();

/[~~]/v;
