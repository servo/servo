
let testNumber = 1;

function testNodeConnector(testFunction, name) {
    let container = document.createElement('div');
    container.appendChild(document.createElement('div'));
    document.body.appendChild(container);

    test(function () {
        var element = define_new_custom_element();
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(container, instance);
        assert_array_equals(element.takeLog().types(), ['connected']);
    }, name + ' must enqueue a connected reaction');

    test(function () {
        var element = define_new_custom_element();
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        var newDoc = document.implementation.createHTMLDocument();
        testFunction(container, instance);
        assert_array_equals(element.takeLog().types(), ['connected']);
        testFunction(newDoc.documentElement, instance);
        assert_array_equals(element.takeLog().types(), ['disconnected', 'adopted', 'connected']);
    }, name + ' must enqueue a disconnected reaction, an adopted reaction, and a connected reaction when the custom element was in another document');

    container.parentNode.removeChild(container);
}

function testNodeDisconnector(testFunction, name) {
    let container = document.createElement('div');
    container.appendChild(document.createElement('div'));
    document.body.appendChild(container);
 
    test(function () {
        var element = define_new_custom_element();
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        container.appendChild(instance);
        assert_array_equals(element.takeLog().types(), ['connected']);
        testFunction(instance, window);
        assert_array_equals(element.takeLog().types(), ['disconnected']);
    }, name + ' must enqueue a disconnected reaction');

    container.parentNode.removeChild(container);
}

function testInsertingMarkup(testFunction, name) {
    let container = document.createElement('div');
    container.appendChild(document.createElement('div'));
    document.body.appendChild(container);

    test(function () {
        var element = define_new_custom_element();
        testFunction(container, `<${element.name}></${element.name}>`);
        assert_array_equals(element.takeLog().types(), ['constructed', 'connected']);
    }, name + ' must enqueue a connected reaction for a newly constructed custom element');

    test(function () {
        var element = define_new_custom_element(['title']);
        testFunction(container, `<${element.name} id="hello" title="hi"></${element.name}>`);
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['constructed', 'attributeChanged', 'connected']);
        assert_attribute_log_entry(logEntries[1], {name: 'title', oldValue: null, newValue: 'hi', namespace: null});
    }, name + ' must enqueue a attributeChanged reaction for a newly constructed custom element');

    container.parentNode.removeChild(container);
}

function testParsingMarkup(testFunction, name) {
    test(function () {
        var element = define_new_custom_element(['id']);
        assert_array_equals(element.takeLog().types(), []);
        var instance = testFunction(document, `<${element.name} id="hello" class="foo"></${element.name}>`);
        assert_equals(Object.getPrototypeOf(instance.querySelector(element.name)), element.class.prototype);
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['constructed', 'attributeChanged']);
        assert_attribute_log_entry(logEntries[1], {name: 'id', oldValue: null, newValue: 'hello', namespace: null});
    }, name + ' must construct a custom element');
}

function testCloner(testFunction, name) {
    let container = document.createElement('div');
    container.appendChild(document.createElement('div'));
    document.body.appendChild(container);

    test(function () {
        var element = define_new_custom_element(['id']);
        var instance = document.createElement(element.name);
        container.appendChild(instance);

        instance.setAttribute('id', 'foo');
        assert_array_equals(element.takeLog().types(), ['constructed', 'connected', 'attributeChanged']);
        var newInstance = testFunction(instance);
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['constructed', 'attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: 'id', oldValue: null, newValue: 'foo', namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when cloning an element with an observed attribute');

    test(function () {
        var element = define_new_custom_element(['id']);
        var instance = document.createElement(element.name);
        container.appendChild(instance);

        instance.setAttribute('lang', 'en');
        assert_array_equals(element.takeLog().types(), ['constructed', 'connected']);
        var newInstance = testFunction(instance);
        assert_array_equals(element.takeLog().types(), ['constructed']);
    }, name + ' must not enqueue an attributeChanged reaction when cloning an element with an unobserved attribute');

    test(function () {
        var element = define_new_custom_element(['title', 'class']);
        var instance = document.createElement(element.name);
        container.appendChild(instance);

        instance.setAttribute('lang', 'en');
        instance.className = 'foo';
        instance.setAttribute('title', 'hello world');
        assert_array_equals(element.takeLog().types(), ['constructed', 'connected', 'attributeChanged', 'attributeChanged']);
        var newInstance = testFunction(instance);
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['constructed', 'attributeChanged', 'attributeChanged']);
        assert_attribute_log_entry(logEntries[1], {name: 'class', oldValue: null, newValue: 'foo', namespace: null});
        assert_attribute_log_entry(logEntries[2], {name: 'title', oldValue: null, newValue: 'hello world', namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when cloning an element only for observed attributes');
}

function testReflectAttributeWithContentValues(jsAttributeName, contentAttributeName, validValue1, contentValue1, validValue2, contentValue2, name) {
    test(function () {
        var element = define_new_custom_element([contentAttributeName]);
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        instance[jsAttributeName] = validValue1;
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);

        assert_attribute_log_entry(logEntries.last(), {name: contentAttributeName, oldValue: null, newValue: contentValue1, namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when adding ' + contentAttributeName + ' content attribute');

    test(function () {
        var element = define_new_custom_element([contentAttributeName]);
        var instance = document.createElement(element.name);
        instance[jsAttributeName] = validValue1;
        assert_array_equals(element.takeLog().types(), ['constructed', 'attributeChanged']);
        instance[jsAttributeName] = validValue2;
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: contentAttributeName, oldValue: contentValue1, newValue: contentValue2, namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when replacing an existing attribute');
}

function testReflectAttribute(jsAttributeName, contentAttributeName, validValue1, validValue2, name) {
    testReflectAttributeWithContentValues(jsAttributeName, contentAttributeName, validValue1, validValue1, validValue2, validValue2, name);
}

function testReflectBooleanAttribute(jsAttributeName, contentAttributeName, name) {
    testReflectAttributeWithContentValues(jsAttributeName, contentAttributeName, true, '', false, null, name);
}

function testAttributeAdder(testFunction, name) {
    test(function () {
        var element = define_new_custom_element(['id']);
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'id', 'foo');
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: 'id', oldValue: null, newValue: 'foo', namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when adding an attribute');

    test(function () {
        var element = define_new_custom_element(['class']);
        var instance = document.createElement(element.name);
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'data-lang', 'en');
        assert_array_equals(element.takeLog().types(), []);
    }, name + ' must not enqueue an attributeChanged reaction when adding an unobserved attribute');

    test(function () {
        var element = define_new_custom_element(['title']);
        var instance = document.createElement(element.name);
        instance.setAttribute('title', 'hello');
        assert_array_equals(element.takeLog().types(), ['constructed', 'attributeChanged']);
        testFunction(instance, 'title', 'world');
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: 'title', oldValue: 'hello', newValue: 'world', namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when replacing an existing attribute');

    test(function () {
        var element = define_new_custom_element([]);
        var instance = document.createElement(element.name);
        instance.setAttribute('data-lang', 'zh');
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'data-lang', 'en');
        assert_array_equals(element.takeLog().types(), []);
    }, name + ' must enqueue an attributeChanged reaction when replacing an existing unobserved attribute');
}

function testAttributeMutator(testFunction, name) {
    test(function () {
        var element = define_new_custom_element(['title']);
        var instance = document.createElement(element.name);
        instance.setAttribute('title', 'hello');
        assert_array_equals(element.takeLog().types(), ['constructed', 'attributeChanged']);
        testFunction(instance, 'title', 'world');
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: 'title', oldValue: 'hello', newValue: 'world', namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when replacing an existing attribute');

    test(function () {
        var element = define_new_custom_element(['class']);
        var instance = document.createElement(element.name);
        instance.setAttribute('data-lang', 'zh');
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'data-lang', 'en');
        assert_array_equals(element.takeLog().types(), []);
    }, name + ' must not enqueue an attributeChanged reaction when replacing an existing unobserved attribute');
}

function testAttributeRemover(testFunction, name, options) {
    if (options && !options.onlyExistingAttribute) {
        test(function () {
            var element = define_new_custom_element(['title']);
            var instance = document.createElement(element.name);
            assert_array_equals(element.takeLog().types(), ['constructed']);
            testFunction(instance, 'title');
            assert_array_equals(element.takeLog().types(), []);
        }, name + ' must not enqueue an attributeChanged reaction when removing an attribute that does not exist');
    }

    test(function () {
        var element = define_new_custom_element([]);
        var instance = document.createElement(element.name);
        instance.setAttribute('data-lang', 'hello');
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'data-lang');
        assert_array_equals(element.takeLog().types(), []);
    }, name + ' must not enqueue an attributeChanged reaction when removing an unobserved attribute');

    test(function () {
        var element = define_new_custom_element(['title']);
        var instance = document.createElement(element.name);
        instance.setAttribute('title', 'hello');
        assert_array_equals(element.takeLog().types(), ['constructed', 'attributeChanged']);
        testFunction(instance, 'title');
        var logEntries = element.takeLog();
        assert_array_equals(logEntries.types(), ['attributeChanged']);
        assert_attribute_log_entry(logEntries.last(), {name: 'title', oldValue: 'hello', newValue: null, namespace: null});
    }, name + ' must enqueue an attributeChanged reaction when removing an existing attribute');

    test(function () {
        var element = define_new_custom_element([]);
        var instance = document.createElement(element.name);
        instance.setAttribute('data-lang', 'ja');
        assert_array_equals(element.takeLog().types(), ['constructed']);
        testFunction(instance, 'data-lang');
        assert_array_equals(element.takeLog().types(), []);
    }, name + ' must not enqueue an attributeChanged reaction when removing an existing unobserved attribute');
}
