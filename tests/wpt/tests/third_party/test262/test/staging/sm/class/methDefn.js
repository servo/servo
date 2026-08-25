// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Method Definitions
info: bugzilla.mozilla.org/show_bug.cgi?id=924672
esid: pending
---*/

// Function definitions.
function syntaxError (script) {
    assert.throws(SyntaxError, function() {
        Function(script);
    });
}


// Tests begin.

syntaxError("{a(){}}");
syntaxError("b = {a(");
syntaxError("b = {a)");
syntaxError("b = {a(}");
syntaxError("b = {a)}");
syntaxError("b = {a()");
syntaxError("b = {a()}");
syntaxError("b = {a(){}");
syntaxError("b = {a){}");
syntaxError("b = {a}}");
syntaxError("b = {a{}}");
syntaxError("b = {a({}}");
syntaxError("b = {a@(){}}");
syntaxError("b = {a() => 0}");
syntaxError("b = {a() void 0}");
syntaxError("b = {a() 1}");
syntaxError("b = {a() false}");

var b;

b = {a(){return 5;}};
assert.sameValue(b.a(), 5);

b = {a(){return "hey";}};
assert.sameValue(b.a(), "hey");

b = {a(){return arguments;}};
assert.sameValue(b.a().length, 0);

b = {a(c){return arguments;}};
assert.sameValue(b.a().length, 0);

b = {a(c){return arguments;}};
assert.sameValue(b.a(1).length, 1);

b = {a(c){return arguments;}};
assert.sameValue(b.a(1)[0], 1);

b = {a(c,d){return arguments;}};
assert.sameValue(b.a(1,2).length, 2);

b = {a(c,d){return arguments;}};
assert.sameValue(b.a(1,2)[0], 1);

b = {a(c,d){return arguments;}};
assert.sameValue(b.a(1,2)[1], 2);

// Methods along with other properties.
syntaxError("b = {,a(){}}");
syntaxError("b = {@,a(){}}");
syntaxError("b = {a(){},@}");
syntaxError("b = {a : 5 , (){}}");
syntaxError("b = {a : 5@ , a(){}}");
syntaxError("b = {a : , a(){}}");
syntaxError("b = {a : 5, a()}}");
syntaxError("b = {a : 5, a({}}");
syntaxError("b = {a : 5, a){}}");
syntaxError("b = {a : 5, a(){}");
var c = "d";
b = { a   : 5,
      [c] : "hey",
      e() {return 6;},
      get f() {return 7;},
      set f(g) {this.h = 9;}
}
assert.sameValue(b.a, 5);
assert.sameValue(b.d, "hey");
assert.sameValue(b.e(), 6);
assert.sameValue(b.f, 7);
assert.sameValue(b.h !== 9, true);
b.f = 15;
assert.sameValue(b.h, 9);


var i = 0;
var a = {
    foo0 : function (){return 0;},
    ["foo" + ++i](){return 1;},
    ["foo" + ++i](){return 2;},
    ["foo" + ++i](){return 3;},
    foo4(){return 4;}
};
assert.sameValue(a.foo0(), 0);
assert.sameValue(a.foo1(), 1);
assert.sameValue(a.foo2(), 2);
assert.sameValue(a.foo3(), 3);
assert.sameValue(a.foo4(), 4);

// Symbols.
var unique_sym = Symbol("1"), registered_sym = Symbol.for("2");
a = { [unique_sym](){return 2;}, [registered_sym](){return 3;} };
assert.sameValue(a[unique_sym](), 2);
assert.sameValue(a[registered_sym](), 3);

// Method characteristics.
a = { b(){ return 4;} };
b = Object.getOwnPropertyDescriptor(a, "b");
assert.sameValue(b.configurable, true);
assert.sameValue(b.enumerable, true);
assert.sameValue(b.writable, true);
assert.sameValue(b.value(), 4);

// prototype property
assert.sameValue(a.b.prototype, undefined);
assert.sameValue(a.b.hasOwnProperty("prototype"), false);

// Defining several methods using eval.
var code = "({";
for (i = 0; i < 1000; i++)
    code += "['foo' + " + i + "]() {return 'ok';}, "
code += "['bar']() {return 'all ok'}});";
var obj = eval(code);
for (i = 0; i < 1000; i++)
    assert.sameValue(obj["foo" + i](), "ok");
assert.sameValue(obj["bar"](), "all ok");

// this
var obj = {
    a : "hey",
    meth(){return this.a;}
}
assert.sameValue(obj.meth(), "hey");

// Inheritance
var obj2 = Object.create(obj);
assert.sameValue(obj2.meth(), "hey");

var obj = {
    a() {
        return "hey";
    }
}
assert.sameValue(obj.a.call(), "hey");

// Duplicates
var obj = {
    meth : 3,
    meth() { return 4; },
    meth() { return 5; }
}
assert.sameValue(obj.meth(), 5);

var obj = {
    meth() { return 4; },
    meth() { return 5; },
    meth : 3
}
assert.sameValue(obj.meth, 3);
assert.throws(TypeError, function() {obj.meth();});

// Strict mode
a = {b(c){"use strict";return c;}};
assert.sameValue(a.b(1), 1);
a = {["b"](c){"use strict";return c;}};
assert.sameValue(a.b(1), 1);

// Allow strict-reserved names as methods in objects.
// (Bug 1124362)
a = { static() { return 4; } };
assert.sameValue(a.static(), 4);

a = { get static() { return 4; } };
assert.sameValue(a.static, 4);

a = { set static(x) { assert.sameValue(x, 4); } };
a.static = 4;

function testStrictMode() {
    "use strict";
    var obj = { static() { return 4; } };
    assert.sameValue(obj.static(), 4);

    obj = { get static() { return 4; } };
    assert.sameValue(obj.static, 4);

    obj = { set static(x) { assert.sameValue(x, 4); } };
    obj.static = 4;
}
testStrictMode();

// Tests provided by benvie in the bug to distinguish from ES5 desugar.
assert.sameValue(({ method() {} }).method.name, "method");
assert.throws(ReferenceError, function() {({ method() { method() } }).method() });
