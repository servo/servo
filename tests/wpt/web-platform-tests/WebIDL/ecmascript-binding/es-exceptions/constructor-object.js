function run_test() {
    test(function() {
        // "There MUST exist a property on the ECMAScript global object whose
        // name is “DOMException” and value is an object called the
        // DOMException constructor object, which provides access to legacy
        // DOMException code constants. The property has the attributes
        // { [[Writable]]: true, [[Enumerable]]: false,
        // [[Configurable]]: true }."
        assert_own_property(self, "DOMException",
                            "self does not have own property \"DOMException\"");
        var desc = Object.getOwnPropertyDescriptor(self, "DOMException");
        assert_false("get" in desc, "self's property \"DOMException\" has getter");
        assert_false("set" in desc, "self's property \"DOMException\" has setter");
        assert_true(desc.writable, "self's property \"DOMException\" is not writable");
        assert_false(desc.enumerable, "self's property \"DOMException\" is enumerable");
        assert_true(desc.configurable, "self's property \"DOMException\" is not configurable");

        // "The DOMException constructor object MUST be a function object but
        // with a [[Prototype]] value of %Error% ([ECMA-262], section 6.1.7.4)."
        assert_equals(Object.getPrototypeOf(self.DOMException), Error,
                      "prototype of self's property \"DOMException\" is not Error");

        // "Its [[Get]] internal property is set as described in ECMA-262
        // section 9.1.8."
        // Not much to test for this.
        // "Its [[Construct]] internal property is set as described in ECMA-262
        // section 19.2.2.3."
        // "Its @@hasInstance property is set as described in ECMA-262 section
        // 19.2.3.8, unless otherwise specified."

        // String() returns something implementation-dependent, because it
        // calls Function#toString.
        assert_class_string(self.DOMException, "Function",
                            "class string of DOMException");

        // "For every legacy code listed in the error names table, there MUST
        // be a property on the DOMException constructor object whose name and
        // value are as indicated in the table. The property has attributes
        // { [[Writable]]: false, [[Enumerable]]: true,
        // [[Configurable]]: false }."
        // See DOMException-constants.html.
    }, "existence and properties of DOMException");

    test(function() {
        assert_own_property(self, "DOMException",
                            "self does not have own property \"DOMException\"");

        // "The DOMException constructor object MUST also have a property named
        // “prototype” with attributes { [[Writable]]: false,
        // [[Enumerable]]: false, [[Configurable]]: false } whose value is an
        // object called the DOMException prototype object. This object also
        // provides access to the legacy code values."
        assert_own_property(self.DOMException, "prototype",
                            'exception "DOMException" does not have own property "prototype"');
        var desc = Object.getOwnPropertyDescriptor(self.DOMException, "prototype");
        assert_false("get" in desc, "DOMException.prototype has getter");
        assert_false("set" in desc, "DOMException.prototype has setter");
        assert_false(desc.writable, "DOMException.prototype is writable");
        assert_false(desc.enumerable, "DOMException.prototype is enumerable");
        assert_false(desc.configurable, "DOMException.prototype is configurable");

        // "The DOMException prototype object MUST have an internal
        // [[Prototype]] property whose value is %ErrorPrototype% ([ECMA-262],
        // section 6.1.7.4)."
        assert_own_property(self, "Error",
                            'should inherit from Error, but self has no such property');
        assert_own_property(self.Error, "prototype",
                            'should inherit from Error, but that object has no "prototype" property');
        assert_equals(Object.getPrototypeOf(self.DOMException.prototype),
                      self.Error.prototype,
                      'prototype of DOMException.prototype is not Error.prototype');

        // "The class string of the DOMException prototype object is
        // “DOMExceptionPrototype”."
        assert_class_string(self.DOMException.prototype, "DOMExceptionPrototype",
                            "class string of DOMException.prototype");
    }, "existence and properties of DOMException.prototype");

    test(function() {
        assert_false(self.DOMException.prototype.hasOwnProperty("name"),
                     "DOMException.prototype should not have an own \"name\" " +
                     "property.");
        assert_false(self.DOMException.prototype.hasOwnProperty("code"),
                     "DOMException.prototype should not have an own \"name\" " +
                     "property.");
    }, "existence of name and code properties on DOMException.prototype");

    test(function() {
        assert_own_property(self, "DOMException",
                            "self does not have own property \"DOMException\"");
        assert_own_property(self.DOMException, "prototype",
                            'interface "DOMException" does not have own property "prototype"');

        // "There MUST be a property named “constructor” on the DOMException
        // prototype object with attributes { [[Writable]]: true,
        // [[Enumerable]]: false, [[Configurable]]: true } and whose value is
        // the DOMException constructor object."
        assert_own_property(self.DOMException.prototype, "constructor",
                            "DOMException" + '.prototype does not have own property "constructor"');
        var desc = Object.getOwnPropertyDescriptor(self.DOMException.prototype, "constructor");
        assert_false("get" in desc, "DOMException.prototype.constructor has getter");
        assert_false("set" in desc, "DOMException.prototype.constructor has setter");
        assert_true(desc.writable, "DOMException.prototype.constructor is not writable");
        assert_false(desc.enumerable, "DOMException.prototype.constructor is enumerable");
        assert_true(desc.configurable, "DOMException.prototype.constructor in not configurable");
        assert_equals(self.DOMException.prototype.constructor, self.DOMException,
                      "DOMException.prototype.constructor is not the same object as DOMException");
    }, "existence and properties of exception interface prototype object's \"constructor\" property");

    done();
}
