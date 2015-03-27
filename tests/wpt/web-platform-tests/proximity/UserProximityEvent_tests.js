(function() {
    //inheritance tests
    test(function() {
        var event = new UserProximityEvent('');
        assert_true(event instanceof window.UserProximityEvent);
    }, 'the event is an instance of UserProximityEvent');

    test(function() {
        var event = new UserProximityEvent('');
        assert_true(event instanceof window.Event);
    }, 'the event inherits from Event');

    //Type attribute tests
    test(function() {
        assert_throws(new TypeError(), function() {
            new UserProximityEvent();
        }, 'First argument is required, so was expecting a TypeError.');
    }, 'Missing type argument');

    test(function() {
        var event = new UserProximityEvent(undefined);
        assert_equals(event.type, 'undefined');
    }, 'Event type set to undefined');

    test(function() {
        var event = new UserProximityEvent(null);
        assert_equals(event.type, 'null');
    }, 'type argument is null');

    test(function() {
        var event = new UserProximityEvent(123);
        assert_equals(event.type, '123');
    }, 'type argument is number');

    test(function() {
        var event = new UserProximityEvent(new Number(123));
        assert_equals(event.type, '123');
    }, 'type argument is Number');

    test(function() {
        var event = new UserProximityEvent([]);
        assert_equals(event.type, '');
    }, 'type argument is array');

    test(function() {
        var event = new UserProximityEvent(new Array());
        assert_equals(event.type, '');
    }, 'type argument is instance of Array');

    test(function() {
        var event = new UserProximityEvent(['t', ['e', ['s', ['t']]]]);
        assert_equals(event.type, 't,e,s,t');
    }, 'type argument is nested array');

    test(function() {
        var event = new UserProximityEvent(Math);
        assert_equals(event.type, '[object Math]');
    }, 'type argument is host object');

    test(function() {
        var event = new UserProximityEvent(true);
        assert_equals(event.type, 'true');
    }, 'type argument is boolean (true)');

    test(function() {
        var event = new UserProximityEvent(new Boolean(true));
        assert_equals(event.type, 'true');
    }, 'type argument is instance of Boolean (true)');

    test(function() {
        var event = new UserProximityEvent(false);
        assert_equals(event.type, 'false');
    }, 'type argument is boolean (false)');

    test(function() {
        var event = new UserProximityEvent(new Boolean(false));
        assert_equals(event.type, 'false');
    }, 'type argument is instance of Boolean (false)');

    test(function() {
        var event = new UserProximityEvent('test');
        assert_equals(event.type, 'test');
    }, 'type argument is string');

    test(function() {
        var event = new UserProximityEvent(new String('test'));
        assert_equals(event.type, 'test');
    }, 'type argument is instance of String');

    test(function() {
        var event = new UserProximityEvent(function test() {});
        assert_regexp_match(event.type, /function test.+{\s?}/);
    }, 'type argument is function');

    test(function() {
        var event = new UserProximityEvent({
            toString: function() {
                return '123';
            }
        });
        assert_equals(event.type, '123');
    }, 'type argument is complext object, with toString method');

    test(function() {
        assert_throws(new TypeError(), function() {
            new UserProximityEvent({
                toString: function() {
                    return function() {}
                }
            });
        });
    }, 'toString is of type function');

    //eventInitDict attribute tests
    test(function() {
        var event = new UserProximityEvent('test', undefined);
        assert_false(event.near);
    }, 'eventInitDict argument sets to undefined');

    test(function() {
        var event = new UserProximityEvent('test', null);
        assert_false(event.near);
    }, 'eventInitDict argument is null');

    test(function() {
        var date = new Date();
        assert_throws(new TypeError(), function() {
            new UserProximityEvent('test', date);
        });
    }, 'eventInitDict argument is Date object');

    test(function() {
        var regexp = /abc/;
        assert_throws(new TypeError(), function() {
            new UserProximityEvent('test', regexp);
        });
    }, 'eventInitDict argument is RegExp object');

    test(function() {
        assert_throws(new TypeError(), function() {
            new UserProximityEvent('test', false);
        });
    }, 'eventInitDict argument is boolean');

    test(function() {
        assert_throws(new TypeError(), function() {
            new UserProximityEvent('test', 123);
        });
    }, 'eventInitDict argument is number');

    test(function() {
        assert_throws(new TypeError(), function() {
            new UserProximityEvent('test', 'hello');
        });
    }, 'eventInitDict argument is string');

    //test readonly attribute boolean near;
    test(function() {
        var event = new UserProximityEvent('test');
        assert_idl_attribute(event, 'near', 'must have attribute near');
    }, 'must have attribute near');

    test(function() {
        var event = new UserProximityEvent('test');
        assert_readonly(event, 'near', 'readonly attribute near');
    }, 'near is readonly');

    test(function() {
        var event = new UserProximityEvent('test');
        assert_false(event.near, 'near initializes to false');
    }, 'near initializes to false');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: false
        });
        assert_false(event.near, 'near set to false');
    }, 'near set to false');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: true
        });
        assert_true(event.near, 'near set to true');
    }, 'near set to true');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: undefined
        });
        assert_false(event.near, 'argument is truthy');
    }, 'near set to undefined');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: null
        });
        assert_false(event.near, 'argument is flasy');
    }, 'near set to null');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: 0
        });
        assert_false(event.near, 'argument is flasy');
    }, 'near set to 0');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: ''
        });
        assert_false(event.near, 'argument is flasy');
    }, 'near set to empty string');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: '\u0020'
        });
        assert_true(event.near, 'argument is truthy');
    }, 'near set to U+0020');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: 1
        });
        assert_true(event.near, 'argument is truthy');
    }, 'near set to 1');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: []
        });
        assert_true(event.near, 'argument is truthy');
    }, 'near set to []');

    test(function() {
        var event = new UserProximityEvent('test', {
            near: {}
        });
        assert_true(event.near, 'argument is truthy');
    }, 'near set to {}');

    test(function() {
        var dict = {
            get test() {
                return false;
            }
        };
        var event = new UserProximityEvent('test', {
            near: dict.test
        });
        assert_false(event.near, 'explict false');
    }, 'near set to object that resolves to false');

    test(function() {
        var desc = 'Expected to find onuserproximity attribute on window object';
        assert_idl_attribute(window, 'onuserproximity', desc);
    }, 'onuserproximity exists');

    test(function() {
        var desc = 'window.onuserproximity must be null';
        assert_equals(window.onuserproximity, null, desc);
    }, 'onuserproximity is null');

    test(function() {
        var desc = 'window.onuserproximity did not accept callable object',
            func = function() {},
            descidl = 'onuserproximity does not exist';
        window.onuserproximity = func;
        assert_equals(window.onuserproximity, func, descidl);
    }, 'onuserproximity exists and can be set to a function');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable as null';
        window.onuserproximity = function() {};
        window.onuserproximity = {};
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat object as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable as null';
        window.onuserproximity = function() {};
        window.onuserproximity = {
            call: 'test'
        };
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat object with non-callable call property as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable (string) as null';
        window.onuserproximity = function() {};
        window.onuserproximity = 'string';
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat string as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable (number) as null';
        window.onuserproximity = function() {};
        window.onuserproximity = 123;
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat number as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable (undefined) as null';
        window.onuserproximity = function() {};
        window.onuserproximity = undefined;
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat undefined as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable (array) as null';
        window.onuserproximity = function() {};
        window.onuserproximity = [];
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat array as null');

    test(function() {
        var desc = 'window.onuserproximity did not treat noncallable host object as null';
        window.onuserproximity = function() {};
        window.onuserproximity = Node;
        assert_equals(window.onuserproximity, null, desc);
    }, 'treat non-callable host object as null');

    //Async tests
    var t = async_test('test if user proximity event received');
    window.addEventListener('userproximity', function(e) {
        t.step(function() {
            var msg = 'expected instance of UserProximityEvent: ';
            assert_true(e instanceof window.UserProximityEvent, msg);
        });
        t.done();
    });

    var t2 = async_test('test if user proximity event received (idl attribute)');
    window.onuserproximity = function(e) {
        t2.step(function() {
            var msg = 'expected instance of UserProximityEvent: ';
            assert_true(e instanceof window.UserProximityEvent, msg);
        });
        t2.done();
    };
})();
