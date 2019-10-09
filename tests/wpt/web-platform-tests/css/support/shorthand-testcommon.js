'use strict';

function test_shorthand_value(property, value, longhands) {
    const stringifiedValue = JSON.stringify(value);

    for (let longhand of Object.keys(longhands).sort()) {
        test(function(){
            var div = document.getElementById('target') || document.createElement('div');
            div.style[property] = "";
            try {
                div.style[property] = value;

                const readValue = div.style[longhand];
                assert_equals(readValue, longhands[longhand], longhand + " should be canonical");

                div.style[longhand] = "";
                div.style[longhand] = readValue;
                assert_equals(div.style[longhand], readValue, "serialization should round-trip");
            } finally {
                div.style[property] = "";
            }
        }, "e.style['" + property + "'] = " + stringifiedValue + " should set " + longhand);
    }

    test(function(){
        var div = document.getElementById('target') || document.createElement('div');
        div.style[property] = "";
        try {
            const expectedLength = div.style.length;
            div.style[property] = value;
            assert_true(CSS.supports(property, value));
            for (let longhand of Object.keys(longhands).sort()) {
                div.style[longhand] = "";
            }
            assert_equals(div.style.length, expectedLength);
        } finally {
            div.style[property] = "";
        }
    }, "e.style['" + property + "'] = " + stringifiedValue + " should not set unrelated longhands");
}
