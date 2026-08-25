/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Parenthesized "destructuring patterns" are not usable as destructuring patterns
info: bugzilla.mozilla.org/show_bug.cgi?id=1146136
esid: pending
features: [class]
---*/

// Don't pollute the top-level script with eval references.
var savedEval = this[String.fromCharCode(101, 118, 97, 108)];

function checkError(code)
{
  function helper(exec, prefix)
  {
    var fullCode = prefix + code;
    assert.throws(SyntaxError, function() {
      exec(fullCode);
    });
  }

  helper(Function, "");
  helper(Function, "'use strict'; ");
  helper(savedEval, "");
  helper(savedEval, "'use strict'; ");
}

// Parenthesized destructuring patterns don't trigger grammar refinement, so we
// get the usual SyntaxError for an invalid assignment target, per
// 12.14.1 second bullet.
checkError("var a, b; ([a, b]) = [1, 2];");
checkError("var a, b; ({a, b}) = { a: 1, b: 2 };");

// *Nested* parenthesized destructuring patterns, on the other hand, do trigger
// grammar refinement.  But subtargets in a destructuring pattern must be
// either object/array literals that match the destructuring pattern refinement
// *or* valid simple assignment targets (or such things with a default, with the
// entire subtarget unparenthesized: |a = 3| is fine, |(a) = 3| is fine for
// destructuring in an expression, |(a = 3)| is forbidden).  Parenthesized
// object/array patterns are neither.  And so 12.14.5.1 third bullet requires an
// early SyntaxError.
checkError("var a, b; ({ a: ({ b: b }) } = { a: { b: 42 } });");
checkError("var a, b; ({ a: { b: (b = 7) } } = { a: {} });");
checkError("var a, b; ({ a: ([b]) } = { a: [42] });");
checkError("var a, b; [(a = 5)] = [1];");
checkError("var a, b; ({ a: (b = 7)} = { b: 1 });");

Function("var a, b; [(a), b] = [1, 2];")();
Function("var a, b; [(a) = 5, b] = [1, 2];")();
Function("var a, b; [(arguments), b] = [1, 2];")();
Function("var a, b; [(arguments) = 5, b] = [1, 2];")();
Function("var a, b; [(eval), b] = [1, 2];")();
Function("var a, b; [(eval) = 5, b] = [1, 2];")();

var repair = {}, demolition = {};

Function("var a, b; [(repair.man), b] = [1, 2];")();
Function("var a, b; [(demolition['man']) = 'motel', b] = [1, 2];")();
Function("var a, b; [(demolition['man' + {}]) = 'motel', b] = [1, 2];")(); // evade constant-folding

Function("var a, b; var obj = { x() { [(super.man), b] = [1, 2]; } };")();
Function("var a, b; var obj = { x() { [(super[8]) = 'motel', b] = [1, 2]; } };")();
Function("var a, b; var obj = { x() { [(super[8 + {}]) = 'motel', b] = [1, 2]; } };")(); // evade constant-folding

// As noted above, when the assignment element has an initializer, the
// assignment element must not be parenthesized.
checkError("var a, b; [(repair.man = 17)] = [1];");
checkError("var a, b; [(demolition['man'] = 'motel')] = [1, 2];");
checkError("var a, b; [(demolition['man' + {}] = 'motel')] = [1];"); // evade constant-folding
checkError("var a, b; var obj = { x() { [(super.man = 5)] = [1]; } };");
checkError("var a, b; var obj = { x() { [(super[8] = 'motel')] = [1]; } };");
checkError("var a, b; var obj = { x() { [(super[8 + {}] = 'motel')] = [1]; } };"); // evade constant-folding

checkError("var a, b; [f() = 'ohai', b] = [1, 2];");
checkError("var a, b; [(f()) = 'kthxbai', b] = [1, 2];");

Function("var a, b; ({ a: (a), b} = { a: 1, b: 2 });")();
Function("var a, b; ({ a: (a) = 5, b} = { a: 1, b: 2 });")();
