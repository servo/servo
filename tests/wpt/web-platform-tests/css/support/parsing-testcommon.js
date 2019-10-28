'use strict';

// serializedValue can be the expected serialization of value,
// or an array of permitted serializations,
// or omitted if value should serialize as value.
function test_valid_value(property, value, serializedValue) {
    if (arguments.length < 3)
        serializedValue = value;

    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.getElementById('target') || document.createElement('div');
        div.style[property] = "";
        div.style[property] = value;
        var readValue = div.style.getPropertyValue(property);
        assert_not_equals(readValue, "", "property should be set");
        if (Array.isArray(serializedValue))
            assert_in_array(readValue, serializedValue, "serialization should be sound");
        else
            assert_equals(readValue, serializedValue, "serialization should be canonical");

        div.style[property] = readValue;
        assert_equals(div.style.getPropertyValue(property), readValue, "serialization should round-trip");

    }, "e.style['" + property + "'] = " + stringifiedValue + " should set the property value");
}

function test_invalid_value(property, value) {
    var stringifiedValue = JSON.stringify(value);

    test(function(){
        var div = document.getElementById('target') || document.createElement('div');
        div.style[property] = "";
        div.style[property] = value;
        assert_equals(div.style.getPropertyValue(property), "");
    }, "e.style['" + property + "'] = " + stringifiedValue + " should not set the property value");
}

// serializedSelector can be the expected serialization of selector,
// or an array of permitted serializations,
// or omitted if value should serialize as selector.
function test_valid_selector(selector, serializedSelector) {
    if (arguments.length < 2)
        serializedSelector = selector;

    const stringifiedSelector = JSON.stringify(selector);

    test(function(){
        document.querySelector(selector);
        assert_true(true, stringifiedSelector + " should not throw in querySelector");

        const style = document.createElement("style");
        document.head.append(style);
        const {sheet} = style;
        document.head.removeChild(style);
        const {cssRules} = sheet;

        assert_equals(cssRules.length, 0, "Sheet should have no rule");
        sheet.insertRule(selector + "{}");
        assert_equals(cssRules.length, 1, "Sheet should have 1 rule");

        const readSelector = cssRules[0].selectorText;
        if (Array.isArray(serializedSelector))
            assert_in_array(readSelector, serializedSelector, "serialization should be sound");
        else
            assert_equals(readSelector, serializedSelector, "serialization should be canonical");

        sheet.deleteRule(0);
        assert_equals(cssRules.length, 0, "Sheet should have no rule");
        sheet.insertRule(readSelector + "{}");
        assert_equals(cssRules.length, 1, "Sheet should have 1 rule");

        assert_equals(cssRules[0].selectorText, readSelector, "serialization should round-trip");
    }, stringifiedSelector + " should be a valid selector");
}

function test_invalid_selector(selector) {
    const stringifiedSelector = JSON.stringify(selector);

    test(function(){
        assert_throws(
          DOMException.SYNTAX_ERR,
          () => document.querySelector(selector),
          stringifiedSelector + " should throw in querySelector");

        const style = document.createElement("style");
        document.head.append(style);
        const {sheet} = style;
        document.head.removeChild(style);

        assert_throws(
          DOMException.SYNTAX_ERR,
          () => sheet.insertRule(selector + "{}"),
          stringifiedSelector + " should throw in insertRule");
    }, stringifiedSelector + " should be an invalid selector");
}
