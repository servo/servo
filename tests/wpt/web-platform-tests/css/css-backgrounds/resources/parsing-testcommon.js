'use strict';

// serializedValue can be the expected serialization of value,
// or an array of permitted serializations,
// or omitted if value should serialize as value.
function test_valid_value(property, value, serializedValue) {
    if (arguments.length < 3)
        serializedValue = value;

    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.createElement('div');
        div.style[property] = value;
        assert_not_equals(div.style[property], "", "property should be set");

        var div = document.createElement('div');
        div.style[property] = value;
        var readValue = div.style[property];
        if (serializedValue instanceof Array)
            assert_true(serializedValue.includes(readValue), "serialization should be sound");
        else
            assert_equals(readValue, serializedValue, "serialization should be canonical");

        div.style[property] = readValue;
        assert_equals(div.style[property], readValue, "serialization should round-trip");

    }, "e.style['" + property + "'] = " + stringifiedValue + " should set the property value");
}

function test_invalid_value(property, value) {
    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.createElement('div');
        div.style[property] = value;
        assert_equals(div.style[property], "");
    }, "e.style['" + property + "'] = " + stringifiedValue + " should not set the property value");
}
