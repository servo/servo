// This file was procedurally generated from the following sources:
// - src/decorator/escaped-accessor.case
// - src/decorator/auto-accessors/cls-decl.template
/*---
description: The accessor modifier of an instance auto-accessor cannot contain a Unicode escape (auto-accessor in a class declaration)
esid: sec-terminal-symbols
features: [decorators]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
---*/
$DONOTEVALUATE();


class C {
  acc\u0065ssor value;
}
