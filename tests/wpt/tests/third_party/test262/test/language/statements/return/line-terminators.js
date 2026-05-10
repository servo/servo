// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    LineTerminator between return and Identifier_opt yields return without
    Identifier_opt
es5id: 12.9_A2
description: Inserting LineTerminator between return and Variable
---*/

assert.sameValue(
  function(){ return
1; }(),
  undefined,
  "#1: LineTerminator(U-000A) between return and Identifier_opt yields return without Identifier_opt"
);

assert.sameValue(
  function(){ return1; }(),
  undefined,
  "#1: LineTerminator(U-000D) between return and Identifier_opt yields return without Identifier_opt"
);


assert.sameValue(
  function(){ return 1; }(),
  undefined,
  "#1: LineTerminator(U-2028) between return and Identifier_opt yields return without Identifier_opt"
);

assert.sameValue(
  function(){ return 1; }(),
  undefined,
  "#1: LineTerminator(U-2029) between return and Identifier_opt yields return without Identifier_opt"
);
