// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts
es6id: 15.1
description: ReturnStatement may not be used directly within global code
info: |
  Syntax

  Script :
    ScriptBodyopt

  ScriptBody :
    StatementList[~Yield, ~Return]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

return;
