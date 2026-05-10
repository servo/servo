// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    ArrowParameters[Yield] :
      BindingIdentifier[?Yield]

    (12.1)
    BindingIdentifier[Yield] :
      Identifier[~Yield] yield

    Identifier :
      IdentifierName but not ReservedWord

    ReservedWord : Keyword

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
var af = switch => 1;
