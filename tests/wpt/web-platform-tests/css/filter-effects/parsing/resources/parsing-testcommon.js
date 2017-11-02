'use strict';

function test_valid_value(property, value, serializedValue) {
    if (arguments.length < 3)
        serializedValue = value;

    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.createElement('div');
        div.style[property] = value;
        assert_not_equals(div.style[property], "");
    }, "e.style['" + property + "'] = " + stringifiedValue + " should set the property value");

    test(function(){
        var div = document.createElement('div');
        div.style[property] = value;
        var readValue = div.style[property];

        if (Array.isArray(serializedValue))
            assert_true(serializedValue.indexOf(readValue) >= 0, '"' + readValue + '" in ' + JSON.stringify(serializedValue));
        else
            assert_equals(readValue, serializedValue);
        div.style[property] = readValue;
        assert_equals(div.style[property], readValue);
    }, "Serialization should round-trip after setting e.style['" + property + "'] = " + stringifiedValue);
}

function test_invalid_value(property, value) {
    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.createElement('div');
        div.style[property] = value;
        assert_equals(div.style[property], "");
    }, "e.style['" + property + "'] = " + stringifiedValue + " should not set the property value");
}
