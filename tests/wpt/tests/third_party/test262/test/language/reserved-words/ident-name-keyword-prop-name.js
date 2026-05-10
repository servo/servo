// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-1-2
description: >
    Allow reserved words as property names at object initialization.
---*/

var tokenCodes = {
    await: 'await',
    break: 'break',
    case: 'case',
    catch: 'catch',
    class: 'class',
    const: 'const',
    continue: 'continue',
    debugger: 'debugger',
    default: 'default',
    delete: 'delete',
    do: 'do',
    else: 'else',
    export: 'export',
    extends: 'extends',
    finally: 'finally',
    for: 'for',
    function: 'function',
    if: 'if',
    import: 'import',
    in: 'in',
    instanceof: 'instanceof',
    new: 'new',
    return: 'return',
    super: 'super',
    switch: 'switch',
    this: 'this',
    throw: 'throw',
    try: 'try',
    typeof: 'typeof',
    var: 'var',
    void: 'void',
    while: 'while',
    with: 'with',
    yield: 'yield',

    enum: 'enum',

    implements: 'implements',
    interface: 'interface',
    package: 'package',
    protected: 'protected',
    private: 'private',
    public: 'public',

    let: 'let',
    static: 'static',
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
}
