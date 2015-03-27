(function() {
    //inheritance tests
    test(function() {
        var event = new DeviceProximityEvent('');
        assert_true(event instanceof window.DeviceProximityEvent);
    }, 'the event is an instance of DeviceProximityEvent');

    test(function() {
        var event = new DeviceProximityEvent('');
        assert_true(event instanceof window.Event);
    }, 'the event inherits from Event');

    //Type attribute tests
    test(function() {
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent();
        }, 'First argument is required, so was expecting a TypeError.');
    }, 'Missing type argument');

    test(function() {
        var event = new DeviceProximityEvent(undefined);
        assert_equals(event.type, 'undefined');
    }, 'Event type set to undefined');

    test(function() {
        var event = new DeviceProximityEvent(null);
        assert_equals(event.type, 'null');
    }, 'type argument is null');

    test(function() {
        var event = new DeviceProximityEvent(123);
        assert_equals(event.type, '123');
    }, 'type argument is number');

    test(function() {
        var event = new DeviceProximityEvent(new Number(123));
        assert_equals(event.type, '123');
    }, 'type argument is Number');

    test(function() {
        var event = new DeviceProximityEvent([]);
        assert_equals(event.type, '');
    }, 'type argument is array');

    test(function() {
        var event = new DeviceProximityEvent(new Array());
        assert_equals(event.type, '');
    }, 'type argument is instance of Array');

    test(function() {
        var event = new DeviceProximityEvent(['t', ['e', ['s', ['t']]]]);
        assert_equals(event.type, 't,e,s,t');
    }, 'type argument is nested array');

    test(function() {
        var event = new DeviceProximityEvent(Math);
        assert_equals(event.type, '[object Math]');
    }, 'type argument is host object');

    test(function() {
        var event = new DeviceProximityEvent(true);
        assert_equals(event.type, 'true');
    }, 'type argument is boolean (true)');

    test(function() {
        var event = new DeviceProximityEvent(new Boolean(true));
        assert_equals(event.type, 'true');
    }, 'type argument is instance of Boolean (true)');

    test(function() {
        var event = new DeviceProximityEvent(false);
        assert_equals(event.type, 'false');
    }, 'type argument is boolean (false)');

    test(function() {
        var event = new DeviceProximityEvent(new Boolean(false));
        assert_equals(event.type, 'false');
    }, 'type argument is instance of Boolean (false)');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_equals(event.type, 'test');
    }, 'type argument is string');

    test(function() {
        var event = new DeviceProximityEvent(new String('test'));
        assert_equals(event.type, 'test');
    }, 'type argument is instance of String');

    test(function() {
        var event = new DeviceProximityEvent(function test() {});
        assert_regexp_match(event.type, /function test.+{\s?}/);
    }, 'type argument is function');

    test(function() {
        var event = new DeviceProximityEvent({
            toString: function() {
                return '123';
            }
        });
        assert_equals(event.type, '123');
    }, 'type argument is complext object, with toString method');

    test(function() {
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent({
                toString: function() {
                    return function() {}
                }
            });
        });
    }, 'toString is of type function');

    //eventInitDict attribute tests
    test(function() {
        var event = new DeviceProximityEvent('test', undefined);
        assert_equals(event.value, Infinity);
        assert_equals(event.min, -Infinity);
        assert_equals(event.max, Infinity);
    }, 'eventInitDict argument sets to undefined');

    test(function() {
        var event = new DeviceProximityEvent('test', null);
        assert_equals(event.value, Infinity);
        assert_equals(event.min, -Infinity);
        assert_equals(event.max, Infinity);
    }, 'eventInitDict argument is null');

    test(function() {
        var date = new Date();
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent('test', date);
        });
    }, 'eventInitDict argument is Date object');

    test(function() {
        var regexp = /abc/;
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent('test', regexp);
        });
    }, 'eventInitDict argument is RegExp object');

    test(function() {
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent('test', false);
        });
    }, 'eventInitDict argument is boolean');

    test(function() {
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent('test', 123);
        });
    }, 'eventInitDict argument is number');

    test(function() {
        assert_throws(new TypeError(), function() {
            new DeviceProximityEvent('test', 'hello');
        });
    }, 'eventInitDict argument is string');

    //test readonly attribute unrestricted double value;
    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_idl_attribute(event, 'value', 'must have attribute value');
    }, 'value attribute exist');

     test(function() {
        var event = new DeviceProximityEvent('test');
        assert_readonly(event, 'value', 'readonly attribute value');
    }, 'value attribute is readonly');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_equals(event.value, Infinity);
    }, 'value initializes to positive Infinity');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: Infinity
        });
        assert_equals(event.value, Infinity);
    }, 'value set to positive Infinity');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: -Infinity
        });
        assert_equals(event.value, -Infinity);
    }, 'value set to negative Infinity');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: 0
        });
        assert_equals(event.value, 0);
    }, 'value set to 0');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: 1
        });
        assert_equals(event.value, 1);
    }, 'value set to 1');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: 0.5
        });
        assert_equals(event.value, 0.5);
    }, 'value set to 0.5');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: false
        });
        assert_equals(event.value, 0, 'value set to false, converts to 0.');
    }, 'value set to false');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: true
        });
        assert_equals(event.value, 1, 'value set to true, converts to 1.');
    }, 'value set to true');


    test(function() {
        var prop = {
            value: undefined
        };
        try {
            var event = new DeviceProximityEvent('test', prop);
            assert_true(isNaN(event.value));
        } catch(e) {
            assert_unreached('error message: ' + e.message);
        }
    }, 'value of undefined resolves to NaN');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: null
        });
        assert_equals(event.value, 0, 'value resolves to 0');
    }, 'value of null resolves to 0');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: ''
        });
        assert_equals(event.value, 0, 'value must resolve to 0');
    }, 'value of empty string must resolve to 0');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: '\u0020'
        });
        assert_equals(event.value, 0, 'value must resolve to 0');
    }, 'value of U+0020 must resolve to 0');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: '\u0020\u0020\u0020\u0020\u0020\u0020'
        });
        assert_equals(event.value, 0, 'value must resolve to 0');
    }, 'value of multiple U+0020 must resolve to 0');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: '\u0020\u0020\u00201234\u0020\u0020\u0020'
        });
        assert_equals(event.value, 1234, 'converts to 1234');
    }, 'value converts to 1234');

    test(function() {
        var event = new DeviceProximityEvent('test', {
            value: []
        });
        assert_equals(event.value, 0, 'converts to 0');
    }, 'value converts to 0');


    test(function() {
        var prop = {
            value: {}
        };
        try {
            var event = new DeviceProximityEvent('test', prop);
            assert_true(isNaN(event.value));
        } catch(e) {
            assert_unreached('error message: ' + e.message);
        }
    }, 'value of {} resolves to NaN');

    test(function() {
        var prop = {
            get value() {
                return NaN;
            }
        };
        try {
            var event = new DeviceProximityEvent('test', prop);
            assert_true(isNaN(event.value));
        } catch(e) {
            assert_unreached('error message: ' + e.message);
        }
    }, 'value resolves to NaN');

    test(function() {
        var prop = {
            get value() {
                return '123';
            }
        };
        var event = new DeviceProximityEvent('test', prop);
        assert_equals(event.value, 123, 'converts to 123');
    }, 'value resolves 123');

    //test readonly attribute unrestricted double min
   test(function() {
        var event = new DeviceProximityEvent('test');
        assert_idl_attribute(event, 'min', 'must have attribute min');
    }, 'min attribute exist');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_readonly(event, 'min', 'readonly attribute min');
    }, 'min attribute is readonly');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_equals(event.min, -Infinity);
    }, 'min initializes to negative Infinity');

    //test readonly attribute unrestricted double max;
    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_idl_attribute(event, 'max', 'must have attribute max');
    }, 'max attribute exist');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_readonly(event, 'max', 'readonly attribute max');
    }, 'max attribute is readonly');

    test(function() {
        var event = new DeviceProximityEvent('test');
        assert_equals(event.max, Infinity);
    }, 'max initializes to positive Infinity');

    //test attribute EventHandler ondeviceproximity;
    test(function() {
        var desc = 'Expected to find ondeviceproximity attribute on window object';
        assert_idl_attribute(window, 'ondeviceproximity', desc);
    }, 'ondeviceproximity exists');

    test(function() {
        var desc = 'window.ondeviceproximity must be null';
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'ondeviceproximity is null');

    test(function() {
        var desc = 'window.ondeviceproximity did not accept callable object',
            func = function() {},
            descidl = 'Expected to find ondeviceproximity attribute on window object';
        assert_idl_attribute(window, 'ondeviceproximity', descidl);
        window.ondeviceproximity = func;
        assert_equals(window.ondeviceproximity, func, desc);
    }, 'ondeviceproximity is set to function');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = {};
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat object as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = {
            call: 'test'
        };
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat object with non-callable call property as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable (string) as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = 'string';
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat string as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable (number) as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = 123;
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat number as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable (undefined) as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = undefined;
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat undefined as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable (array) as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = [];
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat array as null');

    test(function() {
        var desc = 'window.ondeviceproximity did not treat noncallable host object as null';
        window.ondeviceproximity = function() {};
        window.ondeviceproximity = Node;
        assert_equals(window.ondeviceproximity, null, desc);
    }, 'treat non-callable host object as null');

    //Async tests
    var t = async_test('test if device proximity event received');
    window.addEventListener('deviceproximity', function(e) {
        t.step(function() {
            var msg = 'expected instance of DeviceProximityEvent: ';
            assert_true(e instanceof window.DeviceProximityEvent, msg);
        });
        t.done();
    });

    var t2 = async_test('test if user proximity event received');
    window.ondeviceproximity = function(e) {
        t2.step(function() {
            var msg = 'expected instance of DeviceProximityEvent: ';
            assert_true(e instanceof window.DeviceProximityEvent, msg);
        });
        t2.done();
    };
})();
