// This file was procedurally generated from the following sources:
// - src/class-elements/init-err-contains-arguments.case
// - src/class-elements/initializer-error/cls-expr-fields-static-string-literal-name-nested.template
/*---
description: Syntax error if `arguments` used in class field (static string literal ClassElementName)
esid: sec-class-definitions-static-semantics-early-errors
features: [class, class-fields-public, class-static-fields-public]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Static Semantics: Early Errors

      FieldDefinition:
        PropertyNameInitializeropt

      - It is a Syntax Error if ContainsArguments of Initializer is true.

    Static Semantics: ContainsArguments
      IdentifierReference : Identifier

      1. If the StringValue of Identifier is "arguments", return true.
      ...
      For all other grammatical productions, recurse on all nonterminals. If any piece returns true, then return true. Otherwise return false.

---*/

$DONOTEVALUATE();

var C = class {
  static 'x' = () => arguments;
}
