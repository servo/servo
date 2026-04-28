// This file was procedurally generated from the following sources:
// - src/identifier-names/const-escaped.case
// - src/identifier-names/default/obj-assignment-identifier.template
/*---
description: const is a valid identifier name, using escape (IdentifierReference in ObjectAssignmentPattern cannot be a ReservedWord)
esid: prod-AssignmentPattern
features: [destructuring-assignment]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    AssignmentPattern:
      ObjectAssignmentPattern

    ObjectAssignmentPattern:
      { AssignmentPropertyList }

    AssignmentPropertyList:
      AssignmentProperty
      AssignmentPropertyList , AssignmentProperty

    AssignmentProperty:
      IdentifierReference Initializer_opt
      PropertyName : AssignmentElement

    IdentifierReference:
      Identifier
      [~Yield]yield
      [~Await]await

    Identifier:
      IdentifierName but not ReservedWord

---*/


$DONOTEVALUATE();

var x = { \u0063onst } = { const: 42 };
