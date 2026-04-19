// Copyright (C) 2020 Adam Kluball. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Adam Kluball
esid: sec-names-and-keywords
description: zero width joiner is not a valid identifier start
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var \u200D;
