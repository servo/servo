// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-4-2
description: >
    Allow reserved words as property names by accessor functions within an object.
---*/

var test;

var tokenCodes = {
    set await(value) { test = "await"; },
    get await() { return "await"; },
    set break(value) { test = "break"; },
    get break() { return "break"; },
    set case(value) { test = "case"; },
    get case() { return "case"; },
    set catch(value) { test = "catch"; },
    get catch() { return "catch"; },
    set class(value) { test = "class"; },
    get class() { return "class"; },
    set const(value) { test = "const"; },
    get const() { return "const"; },
    set continue(value) { test = "continue"; },
    get continue() { return "continue"; },
    set debugger(value) { test = "debugger"; },
    get debugger() { return "debugger"; },
    set default(value) { test = "default"; },
    get default() { return "default"; },
    set delete(value) { test = "delete"; },
    get delete() { return "delete"; },
    set do(value) { test = "do"; },
    get do() { return "do"; },
    set else(value) { test = "else"; },
    get else() { return "else"; },
    set export(value) { test = "export"; },
    get export() { return "export"; },
    set extends(value) { test = "extends"; },
    get extends() { return "extends"; },
    set finally(value) { test = "finally"; },
    get finally() { return "finally"; },
    set for(value) { test = "for"; },
    get for() { return "for"; },
    set function(value) { test = "function"; },
    get function() { return "function"; },
    set if(value) { test = "if"; },
    get if() { return "if"; },
    set import(value) { test = "import"; },
    get import() { return "import"; },
    set in(value) { test = "in"; },
    get in() { return "in"; },
    set instanceof(value) { test = "instanceof"; },
    get instanceof() { return "instanceof"; },
    set new(value) { test = "new"; },
    get new() { return "new"; },
    set return(value) { test = "return"; },
    get return() { return "return"; },
    set super(value) { test = "super"; },
    get super() { return "super"; },
    set switch(value) { test = "switch"; },
    get switch() { return "switch"; },
    set this(value) { test = "this"; },
    get this() { return "this"; },
    set throw(value) { test = "throw"; },
    get throw() { return "throw"; },
    set try(value) { test = "try"; },
    get try() { return "try"; },
    set typeof(value) { test = "typeof"; },
    get typeof() { return "typeof"; },
    set var(value) { test = "var"; },
    get var() { return "var"; },
    set void(value) { test = "void"; },
    get void() { return "void"; },
    set while(value) { test = "while"; },
    get while() { return "while"; },
    set with(value) { test = "with"; },
    get with() { return "with"; },
    set yield(value) { test = "yield"; },
    get yield() { return "yield"; },

    set enum(value) { test = "enum"; },
    get enum() { return "enum"; },

    set implements(value) { test = "implements"; },
    get implements() { return "implements"; },
    set interface(value) { test = "interface"; },
    get interface() { return "interface"; },
    set package(value) { test = "package"; },
    get package() { return "package"; },
    set private(value) { test = "private"; },
    get private() { return "private"; },
    set protected(value) { test = "protected"; },
    get protected() { return "protected"; },
    set public(value) { test = "public"; },
    get public() { return "public"; },

    set let(value) { test = "let"; },
    get let() { return "let"; },
    set static(value) { test = "static"; },
    get static() { return "static"; },
};

var arr = [
    'await',
    'break',
    'case',
    'catch',
    'class',
    'const',
    'continue',
    'debugger',
    'default',
    'delete',
    'do',
    'else',
    'export',
    'extends',
    'finally',
    'for',
    'function',
    'if',
    'import',
    'in',
    'instanceof',
    'new',
    'return',
    'super',
    'switch',
    'this',
    'throw',
    'try',
    'typeof',
    'var',
    'void',
    'while',
    'with',
    'yield',

    'enum',

    'implements',
    'interface',
    'package',
    'protected',
    'private',
    'public',

    'let',
    'static',
];

for (var i = 0; i < arr.length; ++i) {
    var propertyName = arr[i];

    assert(tokenCodes.hasOwnProperty(propertyName),
           'Property "' + propertyName + '" found');

    assert.sameValue(tokenCodes[propertyName], propertyName,
                     'Property "' + propertyName + '" has correct value');

    tokenCodes[propertyName] = 0;
    assert.sameValue(test, propertyName,
                     'Property "' + propertyName + '" sets correct value');
}
