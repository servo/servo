// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 7.6.1-3-2
description: >
    Allow reserved words as property names by index assignment.
---*/

var tokenCodes = {};

tokenCodes['await'] = 'await';
tokenCodes['break'] = 'break';
tokenCodes['case'] = 'case';
tokenCodes['catch'] = 'catch';
tokenCodes['class'] = 'class';
tokenCodes['const'] = 'const';
tokenCodes['continue'] = 'continue';
tokenCodes['debugger'] = 'debugger';
tokenCodes['default'] = 'default';
tokenCodes['delete'] = 'delete';
tokenCodes['do'] = 'do';
tokenCodes['else'] = 'else';
tokenCodes['export'] = 'export';
tokenCodes['extends'] = 'extends';
tokenCodes['finally'] = 'finally';
tokenCodes['for'] = 'for';
tokenCodes['function'] = 'function';
tokenCodes['if'] = 'if';
tokenCodes['import'] = 'import';
tokenCodes['in'] = 'in';
tokenCodes['instanceof'] = 'instanceof';
tokenCodes['new'] = 'new';
tokenCodes['return'] = 'return';
tokenCodes['super'] = 'super';
tokenCodes['switch'] = 'switch';
tokenCodes['this'] = 'this';
tokenCodes['throw'] = 'throw';
tokenCodes['try'] = 'try';
tokenCodes['typeof'] = 'typeof';
tokenCodes['var'] = 'var';
tokenCodes['void'] = 'void';
tokenCodes['while'] = 'while';
tokenCodes['with'] = 'with';
tokenCodes['yield'] = 'yield';

tokenCodes['enum'] = 'enum';

tokenCodes['implements'] = 'implements';
tokenCodes['interface'] = 'interface';
tokenCodes['package'] = 'package';
tokenCodes['protected'] = 'protected';
tokenCodes['private'] = 'private';
tokenCodes['public'] = 'public';

tokenCodes['let'] = 'let';
tokenCodes['static'] = 'static';


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
