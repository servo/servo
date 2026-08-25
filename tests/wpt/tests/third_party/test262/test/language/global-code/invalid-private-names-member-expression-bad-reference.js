// This file was procedurally generated from the following sources:
// - src/invalid-private-names/member-expression-bad-reference.case
// - src/invalid-private-names/default/top-level-scriptbody.template
/*---
description: bad reference in member expression (Invalid private names should throw a SyntaxError, top level of script body)
esid: sec-static-semantics-early-errors
features: [class-fields-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ScriptBody:StatementList
      It is a Syntax Error if AllPrivateNamesValid of StatementList with an empty List
      as an argument is false unless the source code is eval code that is being
      processed by a direct eval.

    ModuleBody:ModuleItemList
      It is a Syntax Error if AllPrivateNamesValid of ModuleItemList with an empty List
      as an argument is false.


    Static Semantics: AllPrivateNamesValid

    MemberExpression : MemberExpression . PrivateName

    1. If StringValue of PrivateName is in names, return true.
    2. Return false.

    CallExpression : CallExpression . PrivateName

    1. If StringValue of PrivateName is in names, return true.
    2. Return false.

---*/


$DONOTEVALUATE();

something.#x
