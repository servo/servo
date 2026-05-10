// This file was procedurally generated from the following sources:
// - src/invalid-private-names/member-expression-this.case
// - src/invalid-private-names/default/cls-expr-method-heritage.template
/*---
description: this reference in member expression (Invalid private names should throw a SyntaxError, method in inner class expression)
esid: sec-static-semantics-early-errors
features: [class-fields-private, class]
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

    ClassBody : ClassElementList
    1. Let newNames be the concatenation of names with PrivateBoundNames of ClassBody.
    2. Return AllPrivateNamesValid of ClassElementList with the argument newNames.

    For all other grammatical productions, recurse on subexpressions/substatements,
    passing in the names of the caller. If all pieces return true, then return true.
    If any returns false, return false.


    Static Semantics: AllPrivateNamesValid

    MemberExpression : MemberExpression . PrivateName

    1. If StringValue of PrivateName is in names, return true.
    2. Return false.

    CallExpression : CallExpression . PrivateName

    1. If StringValue of PrivateName is in names, return true.
    2. Return false.

---*/


$DONOTEVALUATE();

class Parent {
  #x = 42;
}

var C = class extends Parent {
  m() {
    this.#x
  }
};
