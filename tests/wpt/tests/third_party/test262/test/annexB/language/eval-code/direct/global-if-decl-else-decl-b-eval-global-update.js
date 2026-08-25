// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-update.case
// - src/annex-b-fns/eval-global/direct-if-decl-else-decl-b.template
/*---
description: Variable binding value is updated following evaluation (IfStatement with a declaration in both statement positions in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    b. When the FunctionDeclaration f is evaluated, perform the following steps
       in place of the FunctionDeclaration Evaluation algorithm provided in
       14.1.21:
       i. Let genv be the running execution context's VariableEnvironment.
       ii. Let genvRec be genv's EnvironmentRecord.
       iii. Let benv be the running execution context's LexicalEnvironment.
       iv. Let benvRec be benv's EnvironmentRecord.
       v. Let fobj be ! benvRec.GetBindingValue(F, false).
       vi. Perform ? genvRec.SetMutableBinding(F, fobj, false).
       vii. Return NormalCompletion(empty). 
---*/

eval(
  'if (false) function _f() {} else function f() { return "declaration"; }assert.sameValue(typeof f, "function");\
  assert.sameValue(f(), "declaration");'
);
