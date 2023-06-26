//
// Helper functions for Navigation Timing tests
//

var timingAttributes = [
    'connectEnd',
    'connectStart',
    'domComplete',
    'domContentLoadedEventEnd',
    'domContentLoadedEventStart',
    'domInteractive',
    'domLoading',
    'domainLookupEnd',
    'domainLookupStart',
    'fetchStart',
    'loadEventEnd',
    'loadEventStart',
    'navigationStart',
    'redirectEnd',
    'redirectStart',
    'requestStart',
    'responseEnd',
    'responseStart',
    'unloadEventEnd',
    'unloadEventStart'
];

function test_namespace(child_name, skip_root)
{
    if (skip_root === undefined) {
        var msg = 'window.performance is defined';
        wp_test(function () { assert_not_equals(performanceNamespace, undefined, msg); }, msg);
    }

    if (child_name !== undefined) {
        var msg2 = 'window.performance.' + child_name + ' is defined';
        wp_test(function() { assert_not_equals(performanceNamespace[child_name], undefined, msg2); }, msg2);
    }
}

function test_attribute_exists(parent_name, attribute_name, properties)
{
    var msg = 'window.performance.' + parent_name + '.' + attribute_name + ' is defined.';
    wp_test(function() { assert_not_equals(performanceNamespace[parent_name][attribute_name], undefined, msg); }, msg, properties);
}

function test_enum(parent_name, enum_name, value, properties)
{
    var msg = 'window.performance.' + parent_name + '.' + enum_name + ' is defined.';
    wp_test(function() { assert_not_equals(performanceNamespace[parent_name][enum_name], undefined, msg); }, msg, properties);

    msg = 'window.performance.' + parent_name + '.' + enum_name + ' = ' + value;
    wp_test(function() { assert_equals(performanceNamespace[parent_name][enum_name], value, msg); }, msg, properties);
}

function test_timing_order(attribute_name, greater_than_attribute, properties)
{
    // ensure it's not 0 first
    var msg = "window.performance.timing." + attribute_name + " > 0";
    wp_test(function() { assert_true(performanceNamespace.timing[attribute_name] > 0, msg); }, msg, properties);

    // ensure it's in the right order
    msg = "window.performance.timing." + attribute_name + " >= window.performance.timing." + greater_than_attribute;
    wp_test(function() { assert_true(performanceNamespace.timing[attribute_name] >= performanceNamespace.timing[greater_than_attribute], msg); }, msg, properties);

    // ensure we have at least 5 microseconds difference or it's 0
    msg = "window.performance.timing." + attribute_name + " difference with window.performance.timing." + greater_than_attribute + " is 0 or at least 5 microseconds";
    var diff = performanceNamespace.timing[attribute_name] - performanceNamespace.timing[greater_than_attribute];
    wp_test(function() { assert_true((diff === 0) || ((diff * 1000) >= 5), msg); }, msg, properties);
}

function test_timing_greater_than(attribute_name, greater_than, properties)
{
    var msg = "window.performance.timing." + attribute_name + " > " + greater_than;
    test_greater_than(performanceNamespace.timing[attribute_name], greater_than, msg, properties);
}

function test_timing_equals(attribute_name, equals, msg, properties)
{
    var test_msg = msg || "window.performance.timing." + attribute_name + " == " + equals;
    test_equals(performanceNamespace.timing[attribute_name], equals, test_msg, properties);
}

//
// Non-test related helper functions
//

function sleep_milliseconds(n)
{
    var start = new Date().getTime();
    while (true) {
        if ((new Date().getTime() - start) >= n) break;
    }
}

//
// Common helper functions
//

function test_greater_than(value, greater_than, msg, properties)
{
    wp_test(function () { assert_true(value > greater_than, msg); }, msg, properties);
}

function test_greater_or_equals(value, greater_than, msg, properties)
{
    wp_test(function () { assert_true(value >= greater_than, msg); }, msg, properties);
}

function test_not_equals(value, notequals, msg, properties)
{
    wp_test(function() { assert_not_equals(value, notequals, msg); }, msg, properties);
}
