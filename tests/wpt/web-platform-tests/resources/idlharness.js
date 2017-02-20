/*
Distributed under both the W3C Test Suite License [1] and the W3C
3-clause BSD License [2]. To contribute to a W3C Test Suite, see the
policies and contribution forms [3].

[1] http://www.w3.org/Consortium/Legal/2008/04-testsuite-license
[2] http://www.w3.org/Consortium/Legal/2008/03-bsd-license
[3] http://www.w3.org/2004/10/27-testcases
*/

/* For user documentation see docs/idlharness.md */

/**
 * Notes for people who want to edit this file (not just use it as a library):
 *
 * Most of the interesting stuff happens in the derived classes of IdlObject,
 * especially IdlInterface.  The entry point for all IdlObjects is .test(),
 * which is called by IdlArray.test().  An IdlObject is conceptually just
 * "thing we want to run tests on", and an IdlArray is an array of IdlObjects
 * with some additional data thrown in.
 *
 * The object model is based on what WebIDLParser.js produces, which is in turn
 * based on its pegjs grammar.  If you want to figure out what properties an
 * object will have from WebIDLParser.js, the best way is to look at the
 * grammar:
 *
 *   https://github.com/darobin/webidl.js/blob/master/lib/grammar.peg
 *
 * So for instance:
 *
 *   // interface definition
 *   interface
 *       =   extAttrs:extendedAttributeList? S? "interface" S name:identifier w herit:ifInheritance? w "{" w mem:ifMember* w "}" w ";" w
 *           { return { type: "interface", name: name, inheritance: herit, members: mem, extAttrs: extAttrs }; }
 *
 * This means that an "interface" object will have a .type property equal to
 * the string "interface", a .name property equal to the identifier that the
 * parser found, an .inheritance property equal to either null or the result of
 * the "ifInheritance" production found elsewhere in the grammar, and so on.
 * After each grammatical production is a JavaScript function in curly braces
 * that gets called with suitable arguments and returns some JavaScript value.
 *
 * (Note that the version of WebIDLParser.js we use might sometimes be
 * out-of-date or forked.)
 *
 * The members and methods of the classes defined by this file are all at least
 * briefly documented, hopefully.
 */
(function(){
"use strict";
/// Helpers ///
function constValue (cnt)
//@{
{
    if (cnt.type === "null") return null;
    if (cnt.type === "NaN") return NaN;
    if (cnt.type === "Infinity") return cnt.negative ? -Infinity : Infinity;
    return cnt.value;
}

//@}
function minOverloadLength(overloads)
//@{
{
    if (!overloads.length) {
        return 0;
    }

    return overloads.map(function(attr) {
        return attr.arguments ? attr.arguments.filter(function(arg) {
            return !arg.optional && !arg.variadic;
        }).length : 0;
    })
    .reduce(function(m, n) { return Math.min(m, n); });
}

//@}
function throwOrReject(a_test, operation, fn, obj, args,  message, cb)
//@{
{
    if (operation.idlType.generic !== "Promise") {
        assert_throws(new TypeError(), function() {
            fn.apply(obj, args);
        }, message);
        cb();
    } else {
        try {
            promise_rejects(a_test, new TypeError(), fn.apply(obj, args)).then(cb, cb);
        } catch (e){
            a_test.step(function() {
                assert_unreached("Throws \"" + e + "\" instead of rejecting promise");
                cb();
            });
        }
    }
}

//@}
function awaitNCallbacks(n, cb, ctx)
//@{
{
    var counter = 0;
    return function() {
        counter++;
        if (counter >= n) {
            cb();
        }
    };
}

//@}
var fround =
//@{
(function(){
    if (Math.fround) return Math.fround;

    var arr = new Float32Array(1);
    return function fround(n) {
        arr[0] = n;
        return arr[0];
    };
})();
//@}

/// IdlArray ///
// Entry point
self.IdlArray = function()
//@{
{
    /**
     * A map from strings to the corresponding named IdlObject, such as
     * IdlInterface or IdlException.  These are the things that test() will run
     * tests on.
     */
    this.members = {};

    /**
     * A map from strings to arrays of strings.  The keys are interface or
     * exception names, and are expected to also exist as keys in this.members
     * (otherwise they'll be ignored).  This is populated by add_objects() --
     * see documentation at the start of the file.  The actual tests will be
     * run by calling this.members[name].test_object(obj) for each obj in
     * this.objects[name].  obj is a string that will be eval'd to produce a
     * JavaScript value, which is supposed to be an object implementing the
     * given IdlObject (interface, exception, etc.).
     */
    this.objects = {};

    /**
     * When adding multiple collections of IDLs one at a time, an earlier one
     * might contain a partial interface or implements statement that depends
     * on a later one.  Save these up and handle them right before we run
     * tests.
     *
     * .partials is simply an array of objects from WebIDLParser.js'
     * "partialinterface" production.  .implements maps strings to arrays of
     * strings, such that
     *
     *   A implements B;
     *   A implements C;
     *   D implements E;
     *
     * results in { A: ["B", "C"], D: ["E"] }.
     */
    this.partials = [];
    this["implements"] = {};
};

//@}
IdlArray.prototype.add_idls = function(raw_idls)
//@{
{
    /** Entry point.  See documentation at beginning of file. */
    this.internal_add_idls(WebIDL2.parse(raw_idls));
};

//@}
IdlArray.prototype.add_untested_idls = function(raw_idls)
//@{
{
    /** Entry point.  See documentation at beginning of file. */
    var parsed_idls = WebIDL2.parse(raw_idls);
    for (var i = 0; i < parsed_idls.length; i++)
    {
        parsed_idls[i].untested = true;
        if ("members" in parsed_idls[i])
        {
            for (var j = 0; j < parsed_idls[i].members.length; j++)
            {
                parsed_idls[i].members[j].untested = true;
            }
        }
    }
    this.internal_add_idls(parsed_idls);
};

//@}
IdlArray.prototype.internal_add_idls = function(parsed_idls)
//@{
{
    /**
     * Internal helper called by add_idls() and add_untested_idls().
     * parsed_idls is an array of objects that come from WebIDLParser.js's
     * "definitions" production.  The add_untested_idls() entry point
     * additionally sets an .untested property on each object (and its
     * .members) so that they'll be skipped by test() -- they'll only be
     * used for base interfaces of tested interfaces, return types, etc.
     */
    parsed_idls.forEach(function(parsed_idl)
    {
        if (parsed_idl.type == "interface" && parsed_idl.partial)
        {
            this.partials.push(parsed_idl);
            return;
        }

        if (parsed_idl.type == "implements")
        {
            if (!(parsed_idl.target in this["implements"]))
            {
                this["implements"][parsed_idl.target] = [];
            }
            this["implements"][parsed_idl.target].push(parsed_idl["implements"]);
            return;
        }

        parsed_idl.array = this;
        if (parsed_idl.name in this.members)
        {
            throw "Duplicate identifier " + parsed_idl.name;
        }
        switch(parsed_idl.type)
        {
        case "interface":
            this.members[parsed_idl.name] =
                new IdlInterface(parsed_idl, /* is_callback = */ false);
            break;

        case "dictionary":
            // Nothing to test, but we need the dictionary info around for type
            // checks
            this.members[parsed_idl.name] = new IdlDictionary(parsed_idl);
            break;

        case "typedef":
            this.members[parsed_idl.name] = new IdlTypedef(parsed_idl);
            break;

        case "callback":
            // TODO
            console.log("callback not yet supported");
            break;

        case "enum":
            this.members[parsed_idl.name] = new IdlEnum(parsed_idl);
            break;

        case "callback interface":
            this.members[parsed_idl.name] =
                new IdlInterface(parsed_idl, /* is_callback = */ true);
            break;

        default:
            throw parsed_idl.name + ": " + parsed_idl.type + " not yet supported";
        }
    }.bind(this));
};

//@}
IdlArray.prototype.add_objects = function(dict)
//@{
{
    /** Entry point.  See documentation at beginning of file. */
    for (var k in dict)
    {
        if (k in this.objects)
        {
            this.objects[k] = this.objects[k].concat(dict[k]);
        }
        else
        {
            this.objects[k] = dict[k];
        }
    }
};

//@}
IdlArray.prototype.prevent_multiple_testing = function(name)
//@{
{
    /** Entry point.  See documentation at beginning of file. */
    this.members[name].prevent_multiple_testing = true;
};

//@}
IdlArray.prototype.recursively_get_implements = function(interface_name)
//@{
{
    /**
     * Helper function for test().  Returns an array of things that implement
     * interface_name, so if the IDL contains
     *
     *   A implements B;
     *   B implements C;
     *   B implements D;
     *
     * then recursively_get_implements("A") should return ["B", "C", "D"].
     */
    var ret = this["implements"][interface_name];
    if (ret === undefined)
    {
        return [];
    }
    for (var i = 0; i < this["implements"][interface_name].length; i++)
    {
        ret = ret.concat(this.recursively_get_implements(ret[i]));
        if (ret.indexOf(ret[i]) != ret.lastIndexOf(ret[i]))
        {
            throw "Circular implements statements involving " + ret[i];
        }
    }
    return ret;
};

function exposure_set(object, default_set) {
    var exposed = object.extAttrs.filter(function(a) { return a.name == "Exposed" });
    if (exposed.length > 1 || exposed.length < 0) {
        throw "Unexpected Exposed extended attributes on " + memberName + ": " + exposed;
    }

    if (exposed.length === 0) {
        return default_set;
    }

    var set = exposed[0].rhs.value;
    // Could be a list or a string.
    if (typeof set == "string") {
        set = [ set ];
    }
    return set;
}

function exposed_in(globals) {
    if ('document' in self) {
        return globals.indexOf("Window") >= 0;
    }
    if ('DedicatedWorkerGlobalScope' in self &&
        self instanceof DedicatedWorkerGlobalScope) {
        return globals.indexOf("Worker") >= 0 ||
               globals.indexOf("DedicatedWorker") >= 0;
    }
    if ('SharedWorkerGlobalScope' in self &&
        self instanceof SharedWorkerGlobalScope) {
        return globals.indexOf("Worker") >= 0 ||
               globals.indexOf("SharedWorker") >= 0;
    }
    if ('ServiceWorkerGlobalScope' in self &&
        self instanceof ServiceWorkerGlobalScope) {
        return globals.indexOf("Worker") >= 0 ||
               globals.indexOf("ServiceWorker") >= 0;
    }
    throw "Unexpected global object";
}

//@}
IdlArray.prototype.test = function()
//@{
{
    /** Entry point.  See documentation at beginning of file. */

    // First merge in all the partial interfaces and implements statements we
    // encountered.
    this.partials.forEach(function(parsed_idl)
    {
        if (!(parsed_idl.name in this.members)
        || !(this.members[parsed_idl.name] instanceof IdlInterface))
        {
            throw "Partial interface " + parsed_idl.name + " with no original interface";
        }
        if (parsed_idl.extAttrs)
        {
            parsed_idl.extAttrs.forEach(function(extAttr)
            {
                this.members[parsed_idl.name].extAttrs.push(extAttr);
            }.bind(this));
        }
        parsed_idl.members.forEach(function(member)
        {
            this.members[parsed_idl.name].members.push(new IdlInterfaceMember(member));
        }.bind(this));
    }.bind(this));
    this.partials = [];

    for (var lhs in this["implements"])
    {
        this.recursively_get_implements(lhs).forEach(function(rhs)
        {
            var errStr = lhs + " implements " + rhs + ", but ";
            if (!(lhs in this.members)) throw errStr + lhs + " is undefined.";
            if (!(this.members[lhs] instanceof IdlInterface)) throw errStr + lhs + " is not an interface.";
            if (!(rhs in this.members)) throw errStr + rhs + " is undefined.";
            if (!(this.members[rhs] instanceof IdlInterface)) throw errStr + rhs + " is not an interface.";
            this.members[rhs].members.forEach(function(member)
            {
                this.members[lhs].members.push(new IdlInterfaceMember(member));
            }.bind(this));
        }.bind(this));
    }
    this["implements"] = {};

    Object.getOwnPropertyNames(this.members).forEach(function(memberName) {
        var member = this.members[memberName];
        if (!(member instanceof IdlInterface)) {
            return;
        }

        var globals = exposure_set(member, ["Window"]);
        member.exposed = exposed_in(globals);
        member.exposureSet = globals;
    }.bind(this));

    // Now run test() on every member, and test_object() for every object.
    for (var name in this.members)
    {
        this.members[name].test();
        if (name in this.objects)
        {
            this.objects[name].forEach(function(str)
            {
                this.members[name].test_object(str);
            }.bind(this));
        }
    }
};

//@}
IdlArray.prototype.assert_type_is = function(value, type)
//@{
{
    /**
     * Helper function that tests that value is an instance of type according
     * to the rules of WebIDL.  value is any JavaScript value, and type is an
     * object produced by WebIDLParser.js' "type" production.  That production
     * is fairly elaborate due to the complexity of WebIDL's types, so it's
     * best to look at the grammar to figure out what properties it might have.
     */
    if (type.idlType == "any")
    {
        // No assertions to make
        return;
    }

    if (type.nullable && value === null)
    {
        // This is fine
        return;
    }

    if (type.array)
    {
        // TODO: not supported yet
        return;
    }

    if (type.sequence)
    {
        assert_true(Array.isArray(value), "is not array");
        if (!value.length)
        {
            // Nothing we can do.
            return;
        }
        this.assert_type_is(value[0], type.idlType);
        return;
    }

    if (type.generic === "Promise") {
        assert_true("then" in value, "Attribute with a Promise type has a then property");
        // TODO: Ideally, we would check on project fulfillment
        // that we get the right type
        // but that would require making the type check async
        return;
    }

    if (type.generic === "FrozenArray") {
        assert_true(Array.isArray(value), "Value should be array");
        assert_true(Object.isFrozen(value), "Value should be frozen");
        if (!value.length)
        {
            // Nothing we can do.
            return;
        }
        this.assert_type_is(value[0], type.idlType);
        return;
    }

    type = type.idlType;

    switch(type)
    {
        case "void":
            assert_equals(value, undefined);
            return;

        case "boolean":
            assert_equals(typeof value, "boolean");
            return;

        case "byte":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(-128 <= value && value <= 127, "byte " + value + " not in range [-128, 127]");
            return;

        case "octet":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(0 <= value && value <= 255, "octet " + value + " not in range [0, 255]");
            return;

        case "short":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(-32768 <= value && value <= 32767, "short " + value + " not in range [-32768, 32767]");
            return;

        case "unsigned short":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(0 <= value && value <= 65535, "unsigned short " + value + " not in range [0, 65535]");
            return;

        case "long":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(-2147483648 <= value && value <= 2147483647, "long " + value + " not in range [-2147483648, 2147483647]");
            return;

        case "unsigned long":
            assert_equals(typeof value, "number");
            assert_equals(value, Math.floor(value), "not an integer");
            assert_true(0 <= value && value <= 4294967295, "unsigned long " + value + " not in range [0, 4294967295]");
            return;

        case "long long":
            assert_equals(typeof value, "number");
            return;

        case "unsigned long long":
        case "DOMTimeStamp":
            assert_equals(typeof value, "number");
            assert_true(0 <= value, "unsigned long long is negative");
            return;

        case "float":
            assert_equals(typeof value, "number");
            assert_equals(value, fround(value), "float rounded to 32-bit float should be itself");
            assert_not_equals(value, Infinity);
            assert_not_equals(value, -Infinity);
            assert_not_equals(value, NaN);
            return;

        case "DOMHighResTimeStamp":
        case "double":
            assert_equals(typeof value, "number");
            assert_not_equals(value, Infinity);
            assert_not_equals(value, -Infinity);
            assert_not_equals(value, NaN);
            return;

        case "unrestricted float":
            assert_equals(typeof value, "number");
            assert_equals(value, fround(value), "unrestricted float rounded to 32-bit float should be itself");
            return;

        case "unrestricted double":
            assert_equals(typeof value, "number");
            return;

        case "DOMString":
            assert_equals(typeof value, "string");
            return;

        case "ByteString":
            assert_equals(typeof value, "string");
            assert_regexp_match(value, /^[\x00-\x7F]*$/);
            return;

        case "USVString":
            assert_equals(typeof value, "string");
            assert_regexp_match(value, /^([\x00-\ud7ff\ue000-\uffff]|[\ud800-\udbff][\udc00-\udfff])*$/);
            return;

        case "object":
            assert_true(typeof value == "object" || typeof value == "function", "wrong type: not object or function");
            return;
    }

    if (!(type in this.members))
    {
        throw "Unrecognized type " + type;
    }

    if (this.members[type] instanceof IdlInterface)
    {
        // We don't want to run the full
        // IdlInterface.prototype.test_instance_of, because that could result
        // in an infinite loop.  TODO: This means we don't have tests for
        // NoInterfaceObject interfaces, and we also can't test objects that
        // come from another self.
        assert_true(typeof value == "object" || typeof value == "function", "wrong type: not object or function");
        if (value instanceof Object
        && !this.members[type].has_extended_attribute("NoInterfaceObject")
        && type in self)
        {
            assert_true(value instanceof self[type], "not instanceof " + type);
        }
    }
    else if (this.members[type] instanceof IdlEnum)
    {
        assert_equals(typeof value, "string");
    }
    else if (this.members[type] instanceof IdlDictionary)
    {
        // TODO: Test when we actually have something to test this on
    }
    else if (this.members[type] instanceof IdlTypedef)
    {
        // TODO: Test when we actually have something to test this on
    }
    else
    {
        throw "Type " + type + " isn't an interface or dictionary";
    }
};
//@}

/// IdlObject ///
function IdlObject() {}
IdlObject.prototype.test = function()
//@{
{
    /**
     * By default, this does nothing, so no actual tests are run for IdlObjects
     * that don't define any (e.g., IdlDictionary at the time of this writing).
     */
};

//@}
IdlObject.prototype.has_extended_attribute = function(name)
//@{
{
    /**
     * This is only meaningful for things that support extended attributes,
     * such as interfaces, exceptions, and members.
     */
    return this.extAttrs.some(function(o)
    {
        return o.name == name;
    });
};

//@}

/// IdlDictionary ///
// Used for IdlArray.prototype.assert_type_is
function IdlDictionary(obj)
//@{
{
    /**
     * obj is an object produced by the WebIDLParser.js "dictionary"
     * production.
     */

    /** Self-explanatory. */
    this.name = obj.name;

    /** An array of objects produced by the "dictionaryMember" production. */
    this.members = obj.members;

    /**
     * The name (as a string) of the dictionary type we inherit from, or null
     * if there is none.
     */
    this.base = obj.inheritance;
}

//@}
IdlDictionary.prototype = Object.create(IdlObject.prototype);

/// IdlInterface ///
function IdlInterface(obj, is_callback)
//@{
{
    /**
     * obj is an object produced by the WebIDLParser.js "interface" production.
     */

    /** Self-explanatory. */
    this.name = obj.name;

    /** A back-reference to our IdlArray. */
    this.array = obj.array;

    /**
     * An indicator of whether we should run tests on the interface object and
     * interface prototype object. Tests on members are controlled by .untested
     * on each member, not this.
     */
    this.untested = obj.untested;

    /** An array of objects produced by the "ExtAttr" production. */
    this.extAttrs = obj.extAttrs;

    /** An array of IdlInterfaceMembers. */
    this.members = obj.members.map(function(m){return new IdlInterfaceMember(m); });
    if (this.has_extended_attribute("Unforgeable")) {
        this.members
            .filter(function(m) { return !m["static"] && (m.type == "attribute" || m.type == "operation"); })
            .forEach(function(m) { return m.isUnforgeable = true; });
    }

    /**
     * The name (as a string) of the type we inherit from, or null if there is
     * none.
     */
    this.base = obj.inheritance;

    this._is_callback = is_callback;
}
//@}
IdlInterface.prototype = Object.create(IdlObject.prototype);
IdlInterface.prototype.is_callback = function()
//@{
{
    return this._is_callback;
};
//@}

IdlInterface.prototype.has_constants = function()
//@{
{
    return this.members.some(function(member) {
        return member.type === "const";
    });
};
//@}

IdlInterface.prototype.is_global = function()
//@{
{
    return this.extAttrs.some(function(attribute) {
        return attribute.name === "Global" ||
               attribute.name === "PrimaryGlobal";
    });
};
//@}

IdlInterface.prototype.test = function()
//@{
{
    if (this.has_extended_attribute("NoInterfaceObject"))
    {
        // No tests to do without an instance.  TODO: We should still be able
        // to run tests on the prototype object, if we obtain one through some
        // other means.
        return;
    }

    if (!this.exposed) {
        test(function() {
            assert_false(this.name in self);
        }.bind(this), this.name + " interface: existence and properties of interface object");
        return;
    }

    if (!this.untested)
    {
        // First test things to do with the exception/interface object and
        // exception/interface prototype object.
        this.test_self();
    }
    // Then test things to do with its members (constants, fields, attributes,
    // operations, . . .).  These are run even if .untested is true, because
    // members might themselves be marked as .untested.  This might happen to
    // interfaces if the interface itself is untested but a partial interface
    // that extends it is tested -- then the interface itself and its initial
    // members will be marked as untested, but the members added by the partial
    // interface are still tested.
    this.test_members();
};
//@}

IdlInterface.prototype.test_self = function()
//@{
{
    test(function()
    {
        // This function tests WebIDL as of 2015-01-13.

        // "For every interface that is exposed in a given ECMAScript global
        // environment and:
        // * is a callback interface that has constants declared on it, or
        // * is a non-callback interface that is not declared with the
        //   [NoInterfaceObject] extended attribute,
        // a corresponding property MUST exist on the ECMAScript global object.
        // The name of the property is the identifier of the interface, and its
        // value is an object called the interface object.
        // The property has the attributes { [[Writable]]: true,
        // [[Enumerable]]: false, [[Configurable]]: true }."
        if (this.is_callback() && !this.has_constants()) {
            return;
        }

        // TODO: Should we test here that the property is actually writable
        // etc., or trust getOwnPropertyDescriptor?
        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));
        var desc = Object.getOwnPropertyDescriptor(self, this.name);
        assert_false("get" in desc, "self's property " + format_value(this.name) + " has getter");
        assert_false("set" in desc, "self's property " + format_value(this.name) + " has setter");
        assert_true(desc.writable, "self's property " + format_value(this.name) + " is not writable");
        assert_false(desc.enumerable, "self's property " + format_value(this.name) + " is enumerable");
        assert_true(desc.configurable, "self's property " + format_value(this.name) + " is not configurable");

        if (this.is_callback()) {
            // "The internal [[Prototype]] property of an interface object for
            // a callback interface must be the Function.prototype object."
            assert_equals(Object.getPrototypeOf(self[this.name]), Function.prototype,
                          "prototype of self's property " + format_value(this.name) + " is not Object.prototype");

            return;
        }

        // "The interface object for a given non-callback interface is a
        // function object."
        // "If an object is defined to be a function object, then it has
        // characteristics as follows:"

        // Its [[Prototype]] internal property is otherwise specified (see
        // below).

        // "* Its [[Get]] internal property is set as described in ECMA-262
        //    section 9.1.8."
        // Not much to test for this.

        // "* Its [[Construct]] internal property is set as described in
        //    ECMA-262 section 19.2.2.3."
        // Tested below if no constructor is defined.  TODO: test constructors
        // if defined.

        // "* Its @@hasInstance property is set as described in ECMA-262
        //    section 19.2.3.8, unless otherwise specified."
        // TODO

        // ES6 (rev 30) 19.1.3.6:
        // "Else, if O has a [[Call]] internal method, then let builtinTag be
        // "Function"."
        assert_class_string(self[this.name], "Function", "class string of " + this.name);

        // "The [[Prototype]] internal property of an interface object for a
        // non-callback interface is determined as follows:"
        var prototype = Object.getPrototypeOf(self[this.name]);
        if (this.base) {
            // "* If the interface inherits from some other interface, the
            //    value of [[Prototype]] is the interface object for that other
            //    interface."
            var has_interface_object =
                !this.array
                     .members[this.base]
                     .has_extended_attribute("NoInterfaceObject");
            if (has_interface_object) {
                assert_own_property(self, this.base,
                                    'should inherit from ' + this.base +
                                    ', but self has no such property');
                assert_equals(prototype, self[this.base],
                              'prototype of ' + this.name + ' is not ' +
                              this.base);
            }
        } else {
            // "If the interface doesn't inherit from any other interface, the
            // value of [[Prototype]] is %FunctionPrototype% ([ECMA-262],
            // section 6.1.7.4)."
            assert_equals(prototype, Function.prototype,
                          "prototype of self's property " + format_value(this.name) + " is not Function.prototype");
        }

        if (!this.has_extended_attribute("Constructor")) {
            // "The internal [[Call]] method of the interface object behaves as
            // follows . . .
            //
            // "If I was not declared with a [Constructor] extended attribute,
            // then throw a TypeError."
            assert_throws(new TypeError(), function() {
                self[this.name]();
            }.bind(this), "interface object didn't throw TypeError when called as a function");
            assert_throws(new TypeError(), function() {
                new self[this.name]();
            }.bind(this), "interface object didn't throw TypeError when called as a constructor");
        }
    }.bind(this), this.name + " interface: existence and properties of interface object");

    if (!this.is_callback()) {
        test(function() {
            // This function tests WebIDL as of 2014-10-25.
            // https://heycam.github.io/webidl/#es-interface-call

            assert_own_property(self, this.name,
                                "self does not have own property " + format_value(this.name));

            // "Interface objects for non-callback interfaces MUST have a
            // property named “length” with attributes { [[Writable]]: false,
            // [[Enumerable]]: false, [[Configurable]]: true } whose value is
            // a Number."
            assert_own_property(self[this.name], "length");
            var desc = Object.getOwnPropertyDescriptor(self[this.name], "length");
            assert_false("get" in desc, this.name + ".length has getter");
            assert_false("set" in desc, this.name + ".length has setter");
            assert_false(desc.writable, this.name + ".length is writable");
            assert_false(desc.enumerable, this.name + ".length is enumerable");
            assert_true(desc.configurable, this.name + ".length is not configurable");

            var constructors = this.extAttrs
                .filter(function(attr) { return attr.name == "Constructor"; });
            var expected_length = minOverloadLength(constructors);
            assert_equals(self[this.name].length, expected_length, "wrong value for " + this.name + ".length");
        }.bind(this), this.name + " interface object length");
    }

    if (!this.is_callback() || this.has_constants()) {
        test(function() {
            // This function tests WebIDL as of 2015-11-17.
            // https://heycam.github.io/webidl/#interface-object

            assert_own_property(self, this.name,
                                "self does not have own property " + format_value(this.name));

            // "All interface objects must have a property named “name” with
            // attributes { [[Writable]]: false, [[Enumerable]]: false,
            // [[Configurable]]: true } whose value is the identifier of the
            // corresponding interface."

            assert_own_property(self[this.name], "name");
            var desc = Object.getOwnPropertyDescriptor(self[this.name], "name");
            assert_false("get" in desc, this.name + ".name has getter");
            assert_false("set" in desc, this.name + ".name has setter");
            assert_false(desc.writable, this.name + ".name is writable");
            assert_false(desc.enumerable, this.name + ".name is enumerable");
            assert_true(desc.configurable, this.name + ".name is not configurable");
            assert_equals(self[this.name].name, this.name, "wrong value for " + this.name + ".name");
        }.bind(this), this.name + " interface object name");
    }

    // TODO: Test named constructors if I find any interfaces that have them.

    test(function()
    {
        // This function tests WebIDL as of 2015-01-21.
        // https://heycam.github.io/webidl/#interface-object

        if (this.is_callback() && !this.has_constants()) {
            return;
        }

        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        if (this.is_callback()) {
            assert_false("prototype" in self[this.name],
                         this.name + ' should not have a "prototype" property');
            return;
        }

        // "An interface object for a non-callback interface must have a
        // property named “prototype” with attributes { [[Writable]]: false,
        // [[Enumerable]]: false, [[Configurable]]: false } whose value is an
        // object called the interface prototype object. This object has
        // properties that correspond to the regular attributes and regular
        // operations defined on the interface, and is described in more detail
        // in section 4.5.4 below."
        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');
        var desc = Object.getOwnPropertyDescriptor(self[this.name], "prototype");
        assert_false("get" in desc, this.name + ".prototype has getter");
        assert_false("set" in desc, this.name + ".prototype has setter");
        assert_false(desc.writable, this.name + ".prototype is writable");
        assert_false(desc.enumerable, this.name + ".prototype is enumerable");
        assert_false(desc.configurable, this.name + ".prototype is configurable");

        // Next, test that the [[Prototype]] of the interface prototype object
        // is correct. (This is made somewhat difficult by the existence of
        // [NoInterfaceObject].)
        // TODO: Aryeh thinks there's at least other place in this file where
        //       we try to figure out if an interface prototype object is
        //       correct. Consolidate that code.

        // "The interface prototype object for a given interface A must have an
        // internal [[Prototype]] property whose value is returned from the
        // following steps:
        // "If A is declared with the [Global] or [PrimaryGlobal] extended
        // attribute, and A supports named properties, then return the named
        // properties object for A, as defined in §3.6.4 Named properties
        // object.
        // "Otherwise, if A is declared to inherit from another interface, then
        // return the interface prototype object for the inherited interface.
        // "Otherwise, if A is declared with the [LegacyArrayClass] extended
        // attribute, then return %ArrayPrototype%.
        // "Otherwise, return %ObjectPrototype%.
        if (this.name === "Window") {
            assert_class_string(Object.getPrototypeOf(self[this.name].prototype),
                                'WindowProperties',
                                'Class name for prototype of Window' +
                                '.prototype is not "WindowProperties"');
        } else {
            var inherit_interface, inherit_interface_has_interface_object;
            if (this.base) {
                inherit_interface = this.base;
                inherit_interface_has_interface_object =
                    !this.array
                         .members[inherit_interface]
                         .has_extended_attribute("NoInterfaceObject");
            } else if (this.has_extended_attribute('LegacyArrayClass')) {
                inherit_interface = 'Array';
                inherit_interface_has_interface_object = true;
            } else {
                inherit_interface = 'Object';
                inherit_interface_has_interface_object = true;
            }
            if (inherit_interface_has_interface_object) {
                assert_own_property(self, inherit_interface,
                                    'should inherit from ' + inherit_interface + ', but self has no such property');
                assert_own_property(self[inherit_interface], 'prototype',
                                    'should inherit from ' + inherit_interface + ', but that object has no "prototype" property');
                assert_equals(Object.getPrototypeOf(self[this.name].prototype),
                              self[inherit_interface].prototype,
                              'prototype of ' + this.name + '.prototype is not ' + inherit_interface + '.prototype');
            } else {
                // We can't test that we get the correct object, because this is the
                // only way to get our hands on it. We only test that its class
                // string, at least, is correct.
                assert_class_string(Object.getPrototypeOf(self[this.name].prototype),
                                    inherit_interface + 'Prototype',
                                    'Class name for prototype of ' + this.name +
                                    '.prototype is not "' + inherit_interface + 'Prototype"');
            }
        }

        // "The class string of an interface prototype object is the
        // concatenation of the interface’s identifier and the string
        // “Prototype”."
        assert_class_string(self[this.name].prototype, this.name + "Prototype",
                            "class string of " + this.name + ".prototype");
        // String() should end up calling {}.toString if nothing defines a
        // stringifier.
        if (!this.has_stringifier()) {
            assert_equals(String(self[this.name].prototype), "[object " + this.name + "Prototype]",
                    "String(" + this.name + ".prototype)");
        }
    }.bind(this), this.name + " interface: existence and properties of interface prototype object");

    test(function()
    {
        if (this.is_callback() && !this.has_constants()) {
            return;
        }

        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        if (this.is_callback()) {
            assert_false("prototype" in self[this.name],
                         this.name + ' should not have a "prototype" property');
            return;
        }

        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');

        // "If the [NoInterfaceObject] extended attribute was not specified on
        // the interface, then the interface prototype object must also have a
        // property named “constructor” with attributes { [[Writable]]: true,
        // [[Enumerable]]: false, [[Configurable]]: true } whose value is a
        // reference to the interface object for the interface."
        assert_own_property(self[this.name].prototype, "constructor",
                            this.name + '.prototype does not have own property "constructor"');
        var desc = Object.getOwnPropertyDescriptor(self[this.name].prototype, "constructor");
        assert_false("get" in desc, this.name + ".prototype.constructor has getter");
        assert_false("set" in desc, this.name + ".prototype.constructor has setter");
        assert_true(desc.writable, this.name + ".prototype.constructor is not writable");
        assert_false(desc.enumerable, this.name + ".prototype.constructor is enumerable");
        assert_true(desc.configurable, this.name + ".prototype.constructor in not configurable");
        assert_equals(self[this.name].prototype.constructor, self[this.name],
                      this.name + '.prototype.constructor is not the same object as ' + this.name);
    }.bind(this), this.name + ' interface: existence and properties of interface prototype object\'s "constructor" property');
};

//@}
IdlInterface.prototype.test_member_const = function(member)
//@{
{
    if (!this.has_constants()) {
        throw "Internal error: test_member_const called without any constants";
    }

    test(function()
    {
        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        // "For each constant defined on an interface A, there must be
        // a corresponding property on the interface object, if it
        // exists."
        assert_own_property(self[this.name], member.name);
        // "The value of the property is that which is obtained by
        // converting the constant’s IDL value to an ECMAScript
        // value."
        assert_equals(self[this.name][member.name], constValue(member.value),
                      "property has wrong value");
        // "The property has attributes { [[Writable]]: false,
        // [[Enumerable]]: true, [[Configurable]]: false }."
        var desc = Object.getOwnPropertyDescriptor(self[this.name], member.name);
        assert_false("get" in desc, "property has getter");
        assert_false("set" in desc, "property has setter");
        assert_false(desc.writable, "property is writable");
        assert_true(desc.enumerable, "property is not enumerable");
        assert_false(desc.configurable, "property is configurable");
    }.bind(this), this.name + " interface: constant " + member.name + " on interface object");

    // "In addition, a property with the same characteristics must
    // exist on the interface prototype object."
    test(function()
    {
        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        if (this.is_callback()) {
            assert_false("prototype" in self[this.name],
                         this.name + ' should not have a "prototype" property');
            return;
        }

        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');

        assert_own_property(self[this.name].prototype, member.name);
        assert_equals(self[this.name].prototype[member.name], constValue(member.value),
                      "property has wrong value");
        var desc = Object.getOwnPropertyDescriptor(self[this.name], member.name);
        assert_false("get" in desc, "property has getter");
        assert_false("set" in desc, "property has setter");
        assert_false(desc.writable, "property is writable");
        assert_true(desc.enumerable, "property is not enumerable");
        assert_false(desc.configurable, "property is configurable");
    }.bind(this), this.name + " interface: constant " + member.name + " on interface prototype object");
};


//@}
IdlInterface.prototype.test_member_attribute = function(member)
//@{
  {
    var a_test = async_test(this.name + " interface: attribute " + member.name);
    a_test.step(function()
    {
        if (this.is_callback() && !this.has_constants()) {
            a_test.done()
            return;
        }

        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));
        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');

        if (member["static"]) {
            assert_own_property(self[this.name], member.name,
                "The interface object must have a property " +
                format_value(member.name));
            a_test.done();
        } else if (this.is_global()) {
            assert_own_property(self, member.name,
                "The global object must have a property " +
                format_value(member.name));
            assert_false(member.name in self[this.name].prototype,
                "The prototype object must not have a property " +
                format_value(member.name));

            var getter = Object.getOwnPropertyDescriptor(self, member.name).get;
            assert_equals(typeof(getter), "function",
                          format_value(member.name) + " must have a getter");

            // Try/catch around the get here, since it can legitimately throw.
            // If it does, we obviously can't check for equality with direct
            // invocation of the getter.
            var gotValue;
            var propVal;
            try {
                propVal = self[member.name];
                gotValue = true;
            } catch (e) {
                gotValue = false;
            }
            if (gotValue) {
                assert_equals(propVal, getter.call(undefined),
                              "Gets on a global should not require an explicit this");
            }

            // do_interface_attribute_asserts must be the last thing we do,
            // since it will call done() on a_test.
            this.do_interface_attribute_asserts(self, member, a_test);
        } else {
            assert_true(member.name in self[this.name].prototype,
                "The prototype object must have a property " +
                format_value(member.name));

            if (!member.has_extended_attribute("LenientThis")) {
                if (member.idlType.generic !== "Promise") {
                    assert_throws(new TypeError(), function() {
                        self[this.name].prototype[member.name];
                    }.bind(this), "getting property on prototype object must throw TypeError");
                    // do_interface_attribute_asserts must be the last thing we
                    // do, since it will call done() on a_test.
                    this.do_interface_attribute_asserts(self[this.name].prototype, member, a_test);
                } else {
                    promise_rejects(a_test, new TypeError(),
                                    self[this.name].prototype[member.name])
                        .then(function() {
                            // do_interface_attribute_asserts must be the last
                            // thing we do, since it will call done() on a_test.
                            this.do_interface_attribute_asserts(self[this.name].prototype,
                                                                member, a_test);
                        }.bind(this));
                }
            } else {
                assert_equals(self[this.name].prototype[member.name], undefined,
                              "getting property on prototype object must return undefined");
              // do_interface_attribute_asserts must be the last thing we do,
              // since it will call done() on a_test.
              this.do_interface_attribute_asserts(self[this.name].prototype, member, a_test);
            }

        }
    }.bind(this));
};

//@}
IdlInterface.prototype.test_member_operation = function(member)
//@{
{
    var a_test = async_test(this.name + " interface: operation " + member.name +
                            "(" + member.arguments.map(
                                function(m) {return m.idlType.idlType; } )
                            +")");
    a_test.step(function()
    {
        // This function tests WebIDL as of 2015-12-29.
        // https://heycam.github.io/webidl/#es-operations

        if (this.is_callback() && !this.has_constants()) {
            a_test.done();
            return;
        }

        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        if (this.is_callback()) {
            assert_false("prototype" in self[this.name],
                         this.name + ' should not have a "prototype" property');
            a_test.done();
            return;
        }

        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');

        // "For each unique identifier of an exposed operation defined on the
        // interface, there must exist a corresponding property, unless the
        // effective overload set for that identifier and operation and with an
        // argument count of 0 has no entries."

        // TODO: Consider [Exposed].

        // "The location of the property is determined as follows:"
        var memberHolderObject;
        // "* If the operation is static, then the property exists on the
        //    interface object."
        if (member["static"]) {
            assert_own_property(self[this.name], member.name,
                    "interface object missing static operation");
            memberHolderObject = self[this.name];
        // "* Otherwise, [...] if the interface was declared with the [Global]
        //    or [PrimaryGlobal] extended attribute, then the property exists
        //    on every object that implements the interface."
        } else if (this.is_global()) {
            assert_own_property(self, member.name,
                    "global object missing non-static operation");
            memberHolderObject = self;
        // "* Otherwise, the property exists solely on the interface’s
        //    interface prototype object."
        } else {
            assert_own_property(self[this.name].prototype, member.name,
                    "interface prototype object missing non-static operation");
            memberHolderObject = self[this.name].prototype;
        }
        this.do_member_operation_asserts(memberHolderObject, member, a_test);
    }.bind(this));
};

//@}
IdlInterface.prototype.do_member_operation_asserts = function(memberHolderObject, member, a_test)
//@{
{
    var done = a_test.done.bind(a_test);
    var operationUnforgeable = member.isUnforgeable;
    var desc = Object.getOwnPropertyDescriptor(memberHolderObject, member.name);
    // "The property has attributes { [[Writable]]: B,
    // [[Enumerable]]: true, [[Configurable]]: B }, where B is false if the
    // operation is unforgeable on the interface, and true otherwise".
    assert_false("get" in desc, "property has getter");
    assert_false("set" in desc, "property has setter");
    assert_equals(desc.writable, !operationUnforgeable,
                  "property should be writable if and only if not unforgeable");
    assert_true(desc.enumerable, "property is not enumerable");
    assert_equals(desc.configurable, !operationUnforgeable,
                  "property should be configurable if and only if not unforgeable");
    // "The value of the property is a Function object whose
    // behavior is as follows . . ."
    assert_equals(typeof memberHolderObject[member.name], "function",
                  "property must be a function");
    // "The value of the Function object’s “length” property is
    // a Number determined as follows:
    // ". . .
    // "Return the length of the shortest argument list of the
    // entries in S."
    assert_equals(memberHolderObject[member.name].length,
        minOverloadLength(this.members.filter(function(m) {
            return m.type == "operation" && m.name == member.name;
        })),
        "property has wrong .length");

    // Make some suitable arguments
    var args = member.arguments.map(function(arg) {
        return create_suitable_object(arg.idlType);
    });

    // "Let O be a value determined as follows:
    // ". . .
    // "Otherwise, throw a TypeError."
    // This should be hit if the operation is not static, there is
    // no [ImplicitThis] attribute, and the this value is null.
    //
    // TODO: We currently ignore the [ImplicitThis] case.  Except we manually
    // check for globals, since otherwise we'll invoke window.close().  And we
    // have to skip this test for anything that on the proto chain of "self",
    // since that does in fact have implicit-this behavior.
    if (!member["static"]) {
        var cb;
        if (!this.is_global() &&
            memberHolderObject[member.name] != self[member.name])
        {
            cb = awaitNCallbacks(2, done);
            throwOrReject(a_test, member, memberHolderObject[member.name], null, args,
                          "calling operation with this = null didn't throw TypeError", cb);
        } else {
            cb = awaitNCallbacks(1, done);
        }

        // ". . . If O is not null and is also not a platform object
        // that implements interface I, throw a TypeError."
        //
        // TODO: Test a platform object that implements some other
        // interface.  (Have to be sure to get inheritance right.)
        throwOrReject(a_test, member, memberHolderObject[member.name], {}, args,
                      "calling operation with this = {} didn't throw TypeError", cb);
    } else {
        done();
    }
}

//@}
IdlInterface.prototype.add_iterable_members = function(member)
//@{
{
    this.members.push(new IdlInterfaceMember(
        { type: "operation", name: "entries", idlType: "iterator", arguments: []}));
    this.members.push(new IdlInterfaceMember(
        { type: "operation", name: "keys", idlType: "iterator", arguments: []}));
    this.members.push(new IdlInterfaceMember(
        { type: "operation", name: "values", idlType: "iterator", arguments: []}));
    this.members.push(new IdlInterfaceMember(
        { type: "operation", name: "forEach", idlType: "void",
          arguments:
          [{ name: "callback", idlType: {idlType: "function"}},
           { name: "thisValue", idlType: {idlType: "any"}, optional: true}]}));
};

//@}
IdlInterface.prototype.test_member_iterable = function(member)
//@{
{
    var interfaceName = this.name;
    var isPairIterator = member.idlType instanceof Array;
    test(function()
    {
        var descriptor = Object.getOwnPropertyDescriptor(self[interfaceName].prototype, Symbol.iterator);
        assert_true(descriptor.writable, "property is not writable");
        assert_true(descriptor.configurable, "property is not configurable");
        assert_false(descriptor.enumerable, "property is enumerable");
        assert_equals(self[interfaceName].prototype[Symbol.iterator].name, isPairIterator ? "entries" : "values", "@@iterator function does not have the right name");
    }, "Testing Symbol.iterator property of iterable interface " + interfaceName);

    if (isPairIterator) {
        test(function() {
            assert_equals(self[interfaceName].prototype[Symbol.iterator], self[interfaceName].prototype["entries"], "entries method is not the same as @@iterator");
        }, "Testing pair iterable interface " + interfaceName);
    } else {
        test(function() {
            ["entries", "keys", "values", "forEach", Symbol.Iterator].forEach(function(property) {
                assert_equals(self[interfaceName].prototype[property], Array.prototype[property], property + " function is not the same as Array one");
            });
        }, "Testing value iterable interface " + interfaceName);
    }
};

//@}
IdlInterface.prototype.test_member_stringifier = function(member)
//@{
{
    test(function()
    {
        if (this.is_callback() && !this.has_constants()) {
            return;
        }

        assert_own_property(self, this.name,
                            "self does not have own property " + format_value(this.name));

        if (this.is_callback()) {
            assert_false("prototype" in self[this.name],
                         this.name + ' should not have a "prototype" property');
            return;
        }

        assert_own_property(self[this.name], "prototype",
                            'interface "' + this.name + '" does not have own property "prototype"');

        // ". . . the property exists on the interface prototype object."
        var interfacePrototypeObject = self[this.name].prototype;
        assert_own_property(self[this.name].prototype, "toString",
                "interface prototype object missing non-static operation");

        var stringifierUnforgeable = member.isUnforgeable;
        var desc = Object.getOwnPropertyDescriptor(interfacePrototypeObject, "toString");
        // "The property has attributes { [[Writable]]: B,
        // [[Enumerable]]: true, [[Configurable]]: B }, where B is false if the
        // stringifier is unforgeable on the interface, and true otherwise."
        assert_false("get" in desc, "property has getter");
        assert_false("set" in desc, "property has setter");
        assert_equals(desc.writable, !stringifierUnforgeable,
                      "property should be writable if and only if not unforgeable");
        assert_true(desc.enumerable, "property is not enumerable");
        assert_equals(desc.configurable, !stringifierUnforgeable,
                      "property should be configurable if and only if not unforgeable");
        // "The value of the property is a Function object, which behaves as
        // follows . . ."
        assert_equals(typeof interfacePrototypeObject.toString, "function",
                      "property must be a function");
        // "The value of the Function object’s “length” property is the Number
        // value 0."
        assert_equals(interfacePrototypeObject.toString.length, 0,
            "property has wrong .length");

        // "Let O be the result of calling ToObject on the this value."
        assert_throws(new TypeError(), function() {
            self[this.name].prototype.toString.apply(null, []);
        }, "calling stringifier with this = null didn't throw TypeError");

        // "If O is not an object that implements the interface on which the
        // stringifier was declared, then throw a TypeError."
        //
        // TODO: Test a platform object that implements some other
        // interface.  (Have to be sure to get inheritance right.)
        assert_throws(new TypeError(), function() {
            self[this.name].prototype.toString.apply({}, []);
        }, "calling stringifier with this = {} didn't throw TypeError");
    }.bind(this), this.name + " interface: stringifier");
};

//@}
IdlInterface.prototype.test_members = function()
//@{
{
    for (var i = 0; i < this.members.length; i++)
    {
        var member = this.members[i];
        switch (member.type) {
        case "iterable":
            this.add_iterable_members(member);
            break;
        // TODO: add setlike and maplike handling.
        default:
            break;
        }
    }

    for (var i = 0; i < this.members.length; i++)
    {
        var member = this.members[i];
        if (member.untested) {
            continue;
        }

        if (!exposed_in(exposure_set(member, this.exposureSet))) {
            test(function() {
                // It's not exposed, so we shouldn't find it anywhere.
                assert_false(member.name in self[this.name],
                             "The interface object must not have a property " +
                             format_value(member.name));
                assert_false(member.name in self[this.name].prototype,
                             "The prototype object must not have a property " +
                             format_value(member.name));
            }.bind(this), this.name + " interface: member " + member.name);
            continue;
        }

        switch (member.type) {
        case "const":
            this.test_member_const(member);
            break;

        case "attribute":
            // For unforgeable attributes, we do the checks in
            // test_interface_of instead.
            if (!member.isUnforgeable)
            {
                this.test_member_attribute(member);
            }
            if (member.stringifier) {
                this.test_member_stringifier(member);
            }
            break;

        case "operation":
            // TODO: Need to correctly handle multiple operations with the same
            // identifier.
            // For unforgeable operations, we do the checks in
            // test_interface_of instead.
            if (member.name) {
                if (!member.isUnforgeable)
                {
                    this.test_member_operation(member);
                }
            } else if (member.stringifier) {
                this.test_member_stringifier(member);
            }
            break;

        case "iterable":
            this.test_member_iterable(member);
            break;
        default:
            // TODO: check more member types.
            break;
        }
    }
};

//@}
IdlInterface.prototype.test_object = function(desc)
//@{
{
    var obj, exception = null;
    try
    {
        obj = eval(desc);
    }
    catch(e)
    {
        exception = e;
    }

    var expected_typeof =
        this.members.some(function(member) { return member.legacycaller; })
        ? "function"
        : "object";

    this.test_primary_interface_of(desc, obj, exception, expected_typeof);
    var current_interface = this;
    while (current_interface)
    {
        if (!(current_interface.name in this.array.members))
        {
            throw "Interface " + current_interface.name + " not found (inherited by " + this.name + ")";
        }
        if (current_interface.prevent_multiple_testing && current_interface.already_tested)
        {
            return;
        }
        current_interface.test_interface_of(desc, obj, exception, expected_typeof);
        current_interface = this.array.members[current_interface.base];
    }
};

//@}
IdlInterface.prototype.test_primary_interface_of = function(desc, obj, exception, expected_typeof)
//@{
{
    // We can't easily test that its prototype is correct if there's no
    // interface object, or the object is from a different global environment
    // (not instanceof Object).  TODO: test in this case that its prototype at
    // least looks correct, even if we can't test that it's actually correct.
    if (!this.has_extended_attribute("NoInterfaceObject")
    && (typeof obj != expected_typeof || obj instanceof Object))
    {
        test(function()
        {
            assert_equals(exception, null, "Unexpected exception when evaluating object");
            assert_equals(typeof obj, expected_typeof, "wrong typeof object");
            assert_own_property(self, this.name,
                                "self does not have own property " + format_value(this.name));
            assert_own_property(self[this.name], "prototype",
                                'interface "' + this.name + '" does not have own property "prototype"');

            // "The value of the internal [[Prototype]] property of the
            // platform object is the interface prototype object of the primary
            // interface from the platform object’s associated global
            // environment."
            assert_equals(Object.getPrototypeOf(obj),
                          self[this.name].prototype,
                          desc + "'s prototype is not " + this.name + ".prototype");
        }.bind(this), this.name + " must be primary interface of " + desc);
    }

    // "The class string of a platform object that implements one or more
    // interfaces must be the identifier of the primary interface of the
    // platform object."
    test(function()
    {
        assert_equals(exception, null, "Unexpected exception when evaluating object");
        assert_equals(typeof obj, expected_typeof, "wrong typeof object");
        assert_class_string(obj, this.name, "class string of " + desc);
        if (!this.has_stringifier())
        {
            assert_equals(String(obj), "[object " + this.name + "]", "String(" + desc + ")");
        }
    }.bind(this), "Stringification of " + desc);
};

//@}
IdlInterface.prototype.test_interface_of = function(desc, obj, exception, expected_typeof)
//@{
{
    // TODO: Indexed and named properties, more checks on interface members
    this.already_tested = true;

    for (var i = 0; i < this.members.length; i++)
    {
        var member = this.members[i];
        if (!exposed_in(exposure_set(member, this.exposureSet))) {
            test(function() {
                assert_false(member.name in obj);
            }.bind(this), this.name + "interface: " + desc + 'must not have property "' + member.name + '"');
            continue;
        }
        if (member.type == "attribute" && member.isUnforgeable)
        {
            var a_test = async_test(this.name + " interface: " + desc + ' must have own property "' + member.name + '"');
            a_test.step(function() {
                assert_equals(exception, null, "Unexpected exception when evaluating object");
                assert_equals(typeof obj, expected_typeof, "wrong typeof object");
                // Call do_interface_attribute_asserts last, since it will call a_test.done()
                this.do_interface_attribute_asserts(obj, member, a_test);
            }.bind(this));
        }
        else if (member.type == "operation" &&
                 member.name &&
                 member.isUnforgeable)
        {
            var a_test = async_test(this.name + " interface: " + desc + ' must have own property "' + member.name + '"');
            a_test.step(function()
            {
                assert_equals(exception, null, "Unexpected exception when evaluating object");
                assert_equals(typeof obj, expected_typeof, "wrong typeof object");
                assert_own_property(obj, member.name,
                                    "Doesn't have the unforgeable operation property");
                this.do_member_operation_asserts(obj, member, a_test);
            }.bind(this));
        }
        else if ((member.type == "const"
        || member.type == "attribute"
        || member.type == "operation")
        && member.name)
        {
            test(function()
            {
                assert_equals(exception, null, "Unexpected exception when evaluating object");
                assert_equals(typeof obj, expected_typeof, "wrong typeof object");
                if (!member["static"]) {
                    if (!this.is_global()) {
                        assert_inherits(obj, member.name);
                    } else {
                        assert_own_property(obj, member.name);
                    }

                    if (member.type == "const")
                    {
                        assert_equals(obj[member.name], constValue(member.value));
                    }
                    if (member.type == "attribute")
                    {
                        // Attributes are accessor properties, so they might
                        // legitimately throw an exception rather than returning
                        // anything.
                        var property, thrown = false;
                        try
                        {
                            property = obj[member.name];
                        }
                        catch (e)
                        {
                            thrown = true;
                        }
                        if (!thrown)
                        {
                            this.array.assert_type_is(property, member.idlType);
                        }
                    }
                    if (member.type == "operation")
                    {
                        assert_equals(typeof obj[member.name], "function");
                    }
                }
            }.bind(this), this.name + " interface: " + desc + ' must inherit property "' + member.name + '" with the proper type (' + i + ')');
        }
        // TODO: This is wrong if there are multiple operations with the same
        // identifier.
        // TODO: Test passing arguments of the wrong type.
        if (member.type == "operation" && member.name && member.arguments.length)
        {
            var a_test = async_test( this.name + " interface: calling " + member.name +
            "(" + member.arguments.map(function(m) { return m.idlType.idlType; }) +
            ") on " + desc + " with too few arguments must throw TypeError");
            a_test.step(function()
            {
                assert_equals(exception, null, "Unexpected exception when evaluating object");
                assert_equals(typeof obj, expected_typeof, "wrong typeof object");
                if (!member["static"]) {
                    if (!this.is_global() && !member.isUnforgeable) {
                        assert_inherits(obj, member.name);
                    } else {
                        assert_own_property(obj, member.name);
                    }
                }
                else
                {
                    assert_false(member.name in obj);
                }

                var minLength = minOverloadLength(this.members.filter(function(m) {
                    return m.type == "operation" && m.name == member.name;
                }));
                var args = [];
                var cb = awaitNCallbacks(minLength, a_test.done.bind(a_test));
                for (var i = 0; i < minLength; i++) {
                    throwOrReject(a_test, member, obj[member.name], obj, args,  "Called with " + i + " arguments", cb);

                    args.push(create_suitable_object(member.arguments[i].idlType));
                }
                if (minLength === 0) {
                    cb();
                }
            }.bind(this));
        }
    }
};

//@}
IdlInterface.prototype.has_stringifier = function()
//@{
{
    if (this.members.some(function(member) { return member.stringifier; })) {
        return true;
    }
    if (this.base &&
        this.array.members[this.base].has_stringifier()) {
        return true;
    }
    return false;
};

//@}
IdlInterface.prototype.do_interface_attribute_asserts = function(obj, member, a_test)
//@{
{
    // This function tests WebIDL as of 2015-01-27.
    // TODO: Consider [Exposed].

    // This is called by test_member_attribute() with the prototype as obj if
    // it is not a global, and the global otherwise, and by test_interface_of()
    // with the object as obj.

    var pendingPromises = [];

    // "For each exposed attribute of the interface, whether it was declared on
    // the interface itself or one of its consequential interfaces, there MUST
    // exist a corresponding property. The characteristics of this property are
    // as follows:"

    // "The name of the property is the identifier of the attribute."
    assert_own_property(obj, member.name);

    // "The property has attributes { [[Get]]: G, [[Set]]: S, [[Enumerable]]:
    // true, [[Configurable]]: configurable }, where:
    // "configurable is false if the attribute was declared with the
    // [Unforgeable] extended attribute and true otherwise;
    // "G is the attribute getter, defined below; and
    // "S is the attribute setter, also defined below."
    var desc = Object.getOwnPropertyDescriptor(obj, member.name);
    assert_false("value" in desc, 'property descriptor has value but is supposed to be accessor');
    assert_false("writable" in desc, 'property descriptor has "writable" field but is supposed to be accessor');
    assert_true(desc.enumerable, "property is not enumerable");
    if (member.isUnforgeable)
    {
        assert_false(desc.configurable, "[Unforgeable] property must not be configurable");
    }
    else
    {
        assert_true(desc.configurable, "property must be configurable");
    }


    // "The attribute getter is a Function object whose behavior when invoked
    // is as follows:"
    assert_equals(typeof desc.get, "function", "getter must be Function");

    // "If the attribute is a regular attribute, then:"
    if (!member["static"]) {
        // "If O is not a platform object that implements I, then:
        // "If the attribute was specified with the [LenientThis] extended
        // attribute, then return undefined.
        // "Otherwise, throw a TypeError."
        if (!member.has_extended_attribute("LenientThis")) {
            if (member.idlType.generic !== "Promise") {
                assert_throws(new TypeError(), function() {
                    desc.get.call({});
                }.bind(this), "calling getter on wrong object type must throw TypeError");
            } else {
                pendingPromises.push(
                    promise_rejects(a_test, new TypeError(), desc.get.call({}),
                                    "calling getter on wrong object type must reject the return promise with TypeError"));
            }
        } else {
            assert_equals(desc.get.call({}), undefined,
                          "calling getter on wrong object type must return undefined");
        }
    }

    // "The value of the Function object’s “length” property is the Number
    // value 0."
    assert_equals(desc.get.length, 0, "getter length must be 0");


    // TODO: Test calling setter on the interface prototype (should throw
    // TypeError in most cases).
    if (member.readonly
    && !member.has_extended_attribute("PutForwards")
    && !member.has_extended_attribute("Replaceable"))
    {
        // "The attribute setter is undefined if the attribute is declared
        // readonly and has neither a [PutForwards] nor a [Replaceable]
        // extended attribute declared on it."
        assert_equals(desc.set, undefined, "setter must be undefined for readonly attributes");
    }
    else
    {
        // "Otherwise, it is a Function object whose behavior when
        // invoked is as follows:"
        assert_equals(typeof desc.set, "function", "setter must be function for PutForwards, Replaceable, or non-readonly attributes");

        // "If the attribute is a regular attribute, then:"
        if (!member["static"]) {
            // "If /validThis/ is false and the attribute was not specified
            // with the [LenientThis] extended attribute, then throw a
            // TypeError."
            // "If the attribute is declared with a [Replaceable] extended
            // attribute, then: ..."
            // "If validThis is false, then return."
            if (!member.has_extended_attribute("LenientThis")) {
                assert_throws(new TypeError(), function() {
                    desc.set.call({});
                }.bind(this), "calling setter on wrong object type must throw TypeError");
            } else {
                assert_equals(desc.set.call({}), undefined,
                              "calling setter on wrong object type must return undefined");
            }
        }

        // "The value of the Function object’s “length” property is the Number
        // value 1."
        assert_equals(desc.set.length, 1, "setter length must be 1");
    }

    Promise.all(pendingPromises).then(a_test.done.bind(a_test));
}
//@}

/// IdlInterfaceMember ///
function IdlInterfaceMember(obj)
//@{
{
    /**
     * obj is an object produced by the WebIDLParser.js "ifMember" production.
     * We just forward all properties to this object without modification,
     * except for special extAttrs handling.
     */
    for (var k in obj)
    {
        this[k] = obj[k];
    }
    if (!("extAttrs" in this))
    {
        this.extAttrs = [];
    }

    this.isUnforgeable = this.has_extended_attribute("Unforgeable");
}

//@}
IdlInterfaceMember.prototype = Object.create(IdlObject.prototype);

/// Internal helper functions ///
function create_suitable_object(type)
//@{
{
    /**
     * type is an object produced by the WebIDLParser.js "type" production.  We
     * return a JavaScript value that matches the type, if we can figure out
     * how.
     */
    if (type.nullable)
    {
        return null;
    }
    switch (type.idlType)
    {
        case "any":
        case "boolean":
            return true;

        case "byte": case "octet": case "short": case "unsigned short":
        case "long": case "unsigned long": case "long long":
        case "unsigned long long": case "float": case "double":
        case "unrestricted float": case "unrestricted double":
            return 7;

        case "DOMString":
        case "ByteString":
        case "USVString":
            return "foo";

        case "object":
            return {a: "b"};

        case "Node":
            return document.createTextNode("abc");
    }
    return null;
}
//@}

/// IdlEnum ///
// Used for IdlArray.prototype.assert_type_is
function IdlEnum(obj)
//@{
{
    /**
     * obj is an object produced by the WebIDLParser.js "dictionary"
     * production.
     */

    /** Self-explanatory. */
    this.name = obj.name;

    /** An array of values produced by the "enum" production. */
    this.values = obj.values;

}
//@}

IdlEnum.prototype = Object.create(IdlObject.prototype);

/// IdlTypedef ///
// Used for IdlArray.prototype.assert_type_is
function IdlTypedef(obj)
//@{
{
    /**
     * obj is an object produced by the WebIDLParser.js "typedef"
     * production.
     */

    /** Self-explanatory. */
    this.name = obj.name;

    /** An array of values produced by the "typedef" production. */
    this.values = obj.values;

}
//@}

IdlTypedef.prototype = Object.create(IdlObject.prototype);

}());
// vim: set expandtab shiftwidth=4 tabstop=4 foldmarker=@{,@} foldmethod=marker:
