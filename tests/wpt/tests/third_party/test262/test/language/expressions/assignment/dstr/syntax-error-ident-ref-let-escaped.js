// This file was procedurally generated from the following sources:
// - src/identifier-names/let-escaped.case
// - src/identifier-names/future-reserved-words/obj-assignment-identifier.template
/*---
description: let is a valid identifier name, using escape (IdentifierReference in ObjectAssignmentPattern cannot be a ReservedWord)
esid: prod-AssignmentPattern
features: [destructuring-assignment]
flags: [generated, onlyStrict]
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

var x = { l\u0065t } = { let: 42 };
