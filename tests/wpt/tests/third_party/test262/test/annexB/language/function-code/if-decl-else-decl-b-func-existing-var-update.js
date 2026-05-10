// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-var-update.case
// - src/annex-b-fns/func/if-decl-else-decl-b.template
/*---
description: Variable-scoped binding is updated following evaluation (IfStatement with a declaration in both statement positions in function scope)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    3. When the FunctionDeclaration f is evaluated, perform the following steps
       in place of the FunctionDeclaration Evaluation algorithm provided in
       14.1.21:
       a. Let fenv be the running execution context's VariableEnvironment.
       b. Let fenvRec be fenv's EnvironmentRecord.
       c. Let benv be the running execution context's LexicalEnvironment.
       d. Let benvRec be benv's EnvironmentRecord.
       e. Let fobj be ! benvRec.GetBindingValue(F, false).
       f. Perform ! fenvRec.SetMutableBinding(F, fobj, false).
       g. Return NormalCompletion(empty). 
---*/
var after;

(function() {
  

  if (false) function _f() {} else function f() { return 'function declaration'; }

  after = f;

  var f = 123;
}());

assert.sameValue(typeof after, 'function');
assert.sameValue(after(), 'function declaration');

