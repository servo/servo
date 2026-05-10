// Copyright (C) 2016 Jeff Morrison. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Check that trailing commas are permitted after spread arguments
  in a call expression.
info: http://jeffmo.github.io/es-trailing-function-commas/
author: Jeff Morrison <lbljeffmo@gmail.com>
---*/

function foo() {}
foo(...[],);
