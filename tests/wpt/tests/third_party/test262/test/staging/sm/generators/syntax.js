// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-generators-shell.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// This file was written by Andy Wingo <wingo@igalia.com> and originally
// contributed to V8 as generators-parsing.js, available here:
//
// http://code.google.com/p/v8/source/browse/branches/bleeding_edge/test/mjsunit/harmony/generators-parsing.js

function assertSyntaxError(str) {
    var msg;
    var evil = eval;
    assert.throws(SyntaxError, function() {
        // Non-direct eval.
        evil(str);
    });
}

// Yield statements.
function* g() { yield 3; yield 4; }

// Yield expressions.
function* g() { (yield 3) + (yield 4); }

// Yield without a RHS.
function* g() { yield; }
function* g() { yield }
function* g() {
    yield
}
function* g() { (yield) }
function* g() { [yield] }
function* g() { {yield} }
function* g() { (yield), (yield) }
function* g() { yield; yield }
function* g() { (yield) ? yield : yield }
function* g() {
    (yield)
    ? yield
    : yield
}

// If yield has a RHS, it needs to start on the same line.  The * in a
// yield* counts as starting the RHS.
function* g() {
    yield *
    foo
}
assert.throws(SyntaxError, () => Function("function* g() { yield\n* foo }"));
assertIteratorNext(function*(){
                       yield
                       3
                   }(), undefined)

// A YieldExpression is not a LogicalORExpression.
assert.throws(SyntaxError, () => Function("function* g() { yield ? yield : yield }"));

// You can have a generator in strict mode.
function* g() { "use strict"; yield 3; yield 4; }

// Generators can have return statements also, which internally parse to a kind
// of yield expression.
function* g() { yield 1; return; }
function* g() { yield 1; return 2; }
function* g() { yield 1; return 2; yield "dead"; }

// Generator expression.
(function* () { yield 3; });

// Named generator expression.
(function* g() { yield 3; });

// Generators do not have to contain yield expressions.
function* g() { }

// YieldExpressions can occur in the RHS of a YieldExpression.
function* g() { yield yield 1; }
function* g() { yield 3 + (yield 4); }

// Generator definitions with a name of "yield" are not specifically ruled out
// by the spec, as the `yield' name is outside the generator itself.  However,
// in strict-mode, "yield" is an invalid identifier.
function* yield() { (yield 3) + (yield 4); }
assertSyntaxError("function* yield() { 'use strict'; (yield 3) + (yield 4); }");

// In classic mode, yield is a normal identifier, outside of generators.
function yield(yield) { yield: yield (yield + yield (0)); }

// Yield is always valid as a key in an object literal.
({ yield: 1 });
function* g() { yield ({ yield: 1 }) }
function* g() { yield ({ get yield() { return 1; }}) }

// Yield is a valid property name.
function* g(obj) { yield obj.yield; }

// Checks that yield is a valid label in classic mode, but not valid in a strict
// mode or in generators.
function f() { yield: 1 }
assertSyntaxError("function f() { 'use strict'; yield: 1 }")
assertSyntaxError("function* g() { yield: 1 }")

// Yield is only a keyword in the body of the generator, not in nested
// functions.
function* g() { function f(yield) { yield (yield + yield (0)); } }

// Yield in a generator is not an identifier.
assertSyntaxError("function* g() { yield = 10; }");

// Yield binds very loosely, so this parses as "yield (3 + yield 4)", which is
// invalid.
assertSyntaxError("function* g() { yield 3 + yield 4; }");

// Yield is still a future-reserved-word in strict mode
assertSyntaxError("function f() { 'use strict'; var yield = 13; }");

// The name of the NFE isn't let-bound in F/G, so this is valid.
function f() { (function yield() {}); }
function* g() { (function yield() {}); }

// The name of the NFE is let-bound in the function/generator expression, so this is invalid.
assertSyntaxError("function f() { (function* yield() {}); }");
assertSyntaxError("function* g() { (function* yield() {}); }");

// The name of the declaration is let-bound in F, so this is valid.
function f() { function yield() {} }
function f() { function* yield() {} }

// The name of the declaration is let-bound in G, so this is invalid.
assertSyntaxError("function* g() { function yield() {} }");
assertSyntaxError("function* g() { function* yield() {} }");

// In generators, yield is invalid as a formal argument name.
assertSyntaxError("function* g(yield) { yield (10); }");

