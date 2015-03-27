
# WebIDL 2

[![NPM version](https://badge.fury.io/js/webidl2.png)](http://badge.fury.io/js/webidl2)

Purpose
=======

This is a parser for the [WebIDL](http://dev.w3.org/2006/webapi/WebIDL/) language. If
you don't know what that is, then you probably don't need it. It is meant to be used
both in Node and in the browser (the parser likely works in other JS environments, but
not the test suite).

What of v1?
-----------
There was a previous incarnation of this project. I had written it in the most quick
and dirty manner that was handy because I required it as a dependency in an experiment.
As these things tend to happen, some people started using that, which then had to be
maintained. But since it was not built on solid foundations, it was painful to keep
up to date with the specification, which is a bit of a moving target.

So I started from scratch. Compared to the previous version (which used a parser generator)
this one is about 6x less code (which translates to 4x smaller minified or 2x smaller 
minizipped) and 4x faster. The test suite is reasonably complete (95% coverage), much more
than previously. This version is up to date with WebIDL, rather than a couple years' behind.
It also has *far* better error reporting.

The AST you get from parsing is very similar to the one you got in v1, but some adjustments 
have been made in order to be more systematic, and to map better to what's actually in the spec 
now. If you used v1, you will need to tweak your code but the result ought to be simpler and
you ought to be able to be a fair bit less defensive against irregularities in the way 
information is represented.

Installation
============

Just the usual. For Node:

    npm install webidl2
    
In the browser:

    <script src='webidl2.js'></script>

Documentation
=============

The API to WebIDL2 is trivial: you parse a string of WebIDL and it returns a syntax tree.

Parsing
-------
In Node, that happens with:

    var WebIDL2 = require("webidl2");
    var tree = WebIDL2.parse("string of WebIDL");

In the browser:

    <script src='webidl2.js'></script>
    <script>
      var tree = WebIDL2.parse("string of WebIDL");
    </script>

Errors
------
When there is a syntax error in the WebIDL, it throws an exception object with the following
properties:

* `message`: the error message
* `line`: the line at which the error occurred.
* `input`: a short peek at the text at the point where the error happened
* `tokens`: the five tokens at the point of error, as understood by the tokeniser
  (this is the same content as `input`, but seen from the tokeniser's point of view)

The exception also has a `toString()` method that hopefully should produce a decent
error message.

AST (Abstract Syntax Tree)
--------------------------
The `parse()` method returns a tree object representing the parse tree of the IDL.
Comment and white space are not represented in the AST.

The root of this object is always an array of definitions (where definitions are
any of interfaces, exceptions, callbacks, etc. — anything that can occur at the root
of the IDL).

### IDL Type

This structure is used in many other places (operation return types, argument types, etc.).
It captures a WebIDL type with a number of options. Types look like this and are typically
attached to a field called `idlType`:

    {
        "sequence": false,
        "generic": null,
        "nullable": false,
        "array": false,
        "union": false,
        "idlType": "void"
    }

Where the fields are as follows:

* `sequence`: Boolean indicating whether this is a sequence or not. Deprecated. Use
  `generic` instead.
* `generic`: String indicating the generic type (e.g. "Promise", "sequence"). `null`
  otherwise.
* `nullable`: Boolean indicating whether this is nullable or not.
* `array`: Either `false` to indicate that it is not an array, or a number for the level of
  array nesting.
* `union`: Boolean indicating whether this is a union type or not.
* `idlType`: Can be different things depending on context. In most cases, this will just
  be a string with the type name. But the reason this field isn't called "typeName" is
  because it can take more complex values. If the type is a union, then this contains an
  array of the types it unites. If it is a generic type, it contains the IDL type
  description for the type in the sequence, the eventual value of the promise, etc.

#### Interactions between `nullable` and `array`

A more complex data model for our AST would likely represent `Foo[][][]` as a series of
nested types four levels deep with three anonymous array types eventually containing a 
`Foo` type. But experience shows that such structures are cumbersome to use, and so we
have a simpler model in which the depth of the array is specified with the `array` field.

This is all fine and well, and in the vast majority of cases is actually simpler. But it
does run afoul of cases in which it is necessary to distinguish between `Foo[][][]?`,
`Foo?[][][]`, `Foo[][]?[]`, or even `Foo?[]?[]?[]?`.

For this, when a type is an array type an additional `nullableArray` field is made available
that captures which of the arrays contain nullable elements. It contains booleans that are
true if the given array depth contains nullable elements, and false otherwise (mapping that to
the syntax, and item is true if there is a `?` preceding the `[]`). These examples ought to
clarify the model:

    Foo[][][]?
        -> nullable: true
        -> nullableArray: [false, false, false]
    Foo?[][][]
        -> nullable: false
        -> nullableArray: [true, false, false]
    Foo[][]?[]
        -> nullable: false
        -> nullableArray: [false, false, true]
    Foo?[]?[]?[]?
        -> nullable: true
        -> nullableArray: [true, true, true]

Of particular importance, please note that the overall type is only `nullable` if there is
a `?` at the end.

### Interface
Interfaces look like this:

    {
        "type": "interface",
        "name": "Animal",
        "partial": false,
        "members": [...],
        "inheritance": null,
        "extAttrs": [...]
    },
    {
        "type": "interface",
        "name": "Human",
        "partial": false,
        "members": [...],
        "inheritance": "Animal",
        "extAttrs": [...]
    }

The fields are as follows:

* `type`: Always "interface".
* `name`: The name of the interface
* `partial`: A boolean indicating whether it's a partial interface.
* `members`: An array of interface members (attributes, operations, etc.). Empty if there are none.
* `inheritance`: A string giving the name of an interface this one inherits from, `null` otherwise.
  **NOTE**: In v1 this was an array, but multiple inheritance is no longer supported so this didn't make
  sense.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Callback Interfaces

These are captured by the same structure as [Interfaces](#interface) except that
their `type` field is "callback interface".

### Callback

A callback looks like this:

  {
      "type": "callback",
      "name": "AsyncOperationCallback",
      "idlType": {
          "sequence": false,
          "generic": null,
          "nullable": false,
          "array": false,
          "union": false,
          "idlType": "void"
      },
      "arguments": [...],
      "extAttrs": []
  }

The fields are as follows:

* `type`: Always "callback".
* `name`: The name of the callback.
* `idlType`: An [IDL Type](#idl-type) describing what the callback returns.
* `arguments`: A list of [arguments](#arguments), as in function paramters.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Dictionary

A dictionary looks like this:

    {
        "type": "dictionary",
        "name": "PaintOptions",
        "partial": false,
        "members": [
            {
                "type": "field",
                "name": "fillPattern",
                "idlType": {
                    "sequence": false,
                    "generic": null,
                    "nullable": true,
                    "array": false,
                    "union": false,
                    "idlType": "DOMString"
                },
                "extAttrs": [],
                "default": {
                    "type": "string",
                    "value": "black"
                }
            }
        ],
        "inheritance": null,
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "dictionary".
* `name`: The dictionary name.
* `partial`: Boolean indicating whether it's a partial dictionary.
* `members`: An array of members (see below).
* `inheritance`: A string indicating which dictionary is being inherited from, `null` otherwise.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

All the members are fields as follows:

* `type`: Always "field".
* `name`: The name of the field.
* `idlType`: An [IDL Type](#idl-type) describing what field's type.
* `extAttrs`: A list of [extended attributes](#extended-attributes).
* `default`: A [default value](#default-and-const-values), absent if there is none.

### Exception

An exception looks like this:

    {
        "type": "exception",
        "name": "HierarchyRequestError",
        "members": [
            {
                "type": "field",
                "name": "code",
                "idlType": {
                    "sequence": false,
                    "generic": null,
                    "nullable": false,
                    "array": false,
                    "union": false,
                    "idlType": "unsigned short"
                },
                "extAttrs": []
            }
        ],
        "inheritance": "DOMException",
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "exception".
* `name`: The exception name.
* `members`: An array of members (constants or fields, where fields are described below).
* `inheritance`: A string indicating which exception is being inherited from, `null` otherwise.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

Members that aren't [constants](#constants) have the following fields:

* `type`: Always "field".
* `name`: The field's name.
* `idlType`: An [IDL Type](#idl-type) describing what field's type.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Enum

An enum looks like this:

    {
        "type": "enum",
        "name": "MealType",
        "values": [
            "rice",
            "noodles",
            "other"
        ],
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "enum".
* `name`: The enum's name.
* `value`: An array of values (strings).
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Typedef

A typedef looks like this:

    {
        "type": "typedef",
        "typeExtAttrs": [],
        "idlType": {
            "sequence": true,
            "generic": "sequence",
            "nullable": false,
            "array": false,
            "union": false,
            "idlType": {
                "sequence": false,
                "generic": null,
                "nullable": false,
                "array": false,
                "union": false,
                "idlType": "Point"
            }
        },
        "name": "PointSequence",
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "typedef".
* `name`: The typedef's name.
* `idlType`: An [IDL Type](#idl-type) describing what typedef's type.
* `extAttrs`: A list of [extended attributes](#extended-attributes).
* `typeExtAttrs`: A list of [extended attributes](#extended-attributes) that apply to the 
type rather than to the typedef as a whole.

### Implements

An implements definition looks like this:

    {
        "type": "implements",
        "target": "Node",
        "implements": "EventTarget",
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "implements".
* `target`: The interface that implements another.
* `implements`: The interface that is being implemented by the target.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Operation Member

An operation looks like this:

    {
        "type": "operation",
        "getter": false,
        "setter": false,
        "creator": false,
        "deleter": false,
        "legacycaller": false,
        "static": false,
        "stringifier": false,
        "idlType": {
            "sequence": false,
            "generic": null,
            "nullable": false,
            "array": false,
            "union": false,
            "idlType": "void"
        },
        "name": "intersection",
        "arguments": [
            {
                "optional": false,
                "variadic": true,
                "extAttrs": [],
                "idlType": {
                    "sequence": false,
                    "generic": null,
                    "nullable": false,
                    "array": false,
                    "union": false,
                    "idlType": "long"
                },
                "name": "ints"
            }
        ],
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "operation".
* `getter`: True if a getter operation.
* `setter`: True if a setter operation.
* `creator`: True if a creator operation.
* `deleter`: True if a deleter operation.
* `legacycaller`: True if a legacycaller operation.
* `static`: True if a static operation.
* `stringifier`: True if a stringifier operation.
* `idlType`: An [IDL Type](#idl-type) of what the operation returns. If a stringifier, may be absent.
* `name`: The name of the operation. If a stringifier, may be `null`.
* `arguments`: An array of [arguments](#arguments) for the operation.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Attribute Member

An attribute member looks like this:

    {
        "type": "attribute",
        "static": false,
        "stringifier": false,
        "inherit": false,
        "readonly": false,
        "idlType": {
            "sequence": false,
            "generic": null,
            "nullable": false,
            "array": false,
            "union": false,
            "idlType": "RegExp"
        },
        "name": "regexp",
        "extAttrs": []
    }
    
The fields are as follows:

* `type`: Always "attribute".
* `name`: The attribute's name.
* `static`: True if it's a static attribute.
* `stringifier`: True if it's a stringifier attribute.
* `inherit`: True if it's an inherit attribute.
* `readonly`: True if it's a read-only attribute.
* `idlType`: An [IDL Type](#idl-type) for the attribute.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Constant Member

A constant member looks like this:

    {
        "type": "const",
        "nullable": false,
        "idlType": "boolean",
        "name": "DEBUG",
        "value": {
            "type": "boolean",
            "value": false
        },
        "extAttrs": []
    }

The fields are as follows:

* `type`: Always "const".
* `nullable`: Whether its type is nullable.
* `idlType`: The type of the constant (a simple type, the type name).
* `name`: The name of the constant.
* `value`: The constant value as described by [Const Values](#default-and-const-values)
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Serializer Member

Serializers come in many shapes, which are best understood by looking at the
examples below that map the IDL to the produced AST.

    // serializer;
    {
        "type": "serializer",
        "extAttrs": []
    }

    // serializer DOMString serialize();
    {
        "type": "serializer",
        "idlType": {
            "sequence": false,
            "generic": null,
            "nullable": false,
            "array": false,
            "union": false,
            "idlType": "DOMString"
        },
        "operation": {
            "name": "serialize",
            "arguments": []
        },
        "extAttrs": []
    }

    // serializer = { from, to, amount, description };
    {
        "type": "serializer",
        "patternMap": true,
        "names": [
            "from",
            "to",
            "amount",
            "description"
        ],
        "extAttrs": []
    }

    // serializer = number;
    {
        "type": "serializer",
        "name": "number",
        "extAttrs": []
    }

    // serializer = [ name, number ];
    {
        "type": "serializer",
        "patternList": true,
        "names": [
            "name",
            "number"
        ],
        "extAttrs": []
    }

The common fields are as follows:

* `type`: Always "serializer".
* `extAttrs`: A list of [extended attributes](#extended-attributes).

For a simple serializer, that's all there is. If the serializer is an operation, it will
have:

* `idlType`: An [IDL Type](#idl-type) describing what the serializer returns.
* `operation`: An object with the following fields:
    * `name`: The name of the operation.
    * `arguments`: An array of [arguments](#arguments) for the operation.

If the serializer is a pattern map:

* `patternMap`: Always true.
* `names`: An array of names in the pattern map.

If the serializer is a pattern list:

* `patternList`: Always true.
* `names`: An array of names in the pattern list.

Finally, if the serializer is a named serializer:

* `name`: The serializer's name.

### Iterator Member

Iterator members look like this

    {
        "type": "iterator",
        "getter": false,
        "setter": false,
        "creator": false,
        "deleter": false,
        "legacycaller": false,
        "static": false,
        "stringifier": false,
        "idlType": {
            "sequence": false,
            "generic": null,
            "nullable": false,
            "array": false,
            "union": false,
            "idlType": "Session2"
        },
        "iteratorObject": "SessionIterator",
        "extAttrs": []
    }

* `type`: Always "iterator".
* `iteratorObject`: The string on the right-hand side; absent if there isn't one.
* the rest: same as on [operations](#operation-member).

### Arguments

The arguments (e.g. for an operation) look like this:

    "arguments": [
        {
            "optional": false,
            "variadic": true,
            "extAttrs": [],
            "idlType": {
                "sequence": false,
                "generic": null,
                "nullable": false,
                "array": false,
                "union": false,
                "idlType": "long"
            },
            "name": "ints"
        }
    ]

The fields are as follows:

* `optional`: True if the argument is optional.
* `variadic`: True if the argument is variadic.
* `idlType`: An [IDL Type](#idl-type) describing the type of the argument.
* `name`: The argument's name.
* `extAttrs`: A list of [extended attributes](#extended-attributes).

### Extended Attributes

Extended attributes are arrays of items that look like this:

    "extAttrs": [
        {
            "name": "TreatNullAs",
            "arguments": null,
            "rhs": {
                "type": "identifier",
                "value": "EmptyString"
            }
        }
    ]

The fields are as follows:

* `name`: The extended attribute's name.
* `arguments`: If the extended attribute takes arguments (e.g. `[Foo()]`) or if
  its right-hand side does (e.g. `[NamedConstructor=Name(DOMString blah)]`) they
  are listed here. Note that an empty arguments list will produce an empty array,
  whereas the lack thereof will yield a `null`. If there is an `rhs` field then
  they are the right-hand side's arguments, otherwise they apply to the extended
  attribute directly.
* `rhs`: If there is a right-hand side, this will capture its `type` (always
  "identifier" in practice, though it may be extended in the future) and its
  `value`.
* `typePair`: If the extended attribute is a `MapClass` this will capture the
  map's key type and value type respectively.

### Default and Const Values

Dictionary fields and operation arguments can take default values, and constants take
values, all of which have the following fields:

* `type`: One of string, number, boolean, null, Infinity, or NaN.

For string, number, and boolean:

* `value`: The value of the given type.

For Infinity:

* `negative`: Boolean indicating whether this is negative Infinity or not.


Testing
=======

In order to run the tests you need to ensure that the widlproc submodule inside `test` is
initialised and up to date:

    git submodule init
    git submodule update

Running
-------
The test runs with mocha and expect.js. Normally, running mocha in the root directory
should be enough once you're set up.

Coverage
--------
Current test coverage, as documented in `coverage.html`, is 95%. You can run your own
coverage analysis with:

    jscoverage lib lib-cov
    
That will create the lib-cov directory with instrumented code; the test suite knows
to use that if needed. You can then run the tests with:

    JSCOV=1 mocha --reporter html-cov > coverage.html

Note that I've been getting weirdly overescaped results from the html-cov reporter,
so you might wish to try this instead:

    JSCOV=1 mocha  --reporter html-cov | sed "s/&lt;/</g" | sed "s/&gt;/>/g" | sed "s/&quot;/\"/g" > coverage.html

Browser tests
-------------
In order to test in the browser, get inside `test/web` and run `make-web-tests.js`. This
will generate a `browser-tests.html` file that you can open in a browser. As of this
writing tests pass in the latest Firefox, Chrome, Opera, and Safari. Testing on IE
and older versions will happen progressively.

TODO
====

* add some tests to address coverage limitations
* add a push API for processors that need to process things like comments
