/*
Distributed under both the W3C Test Suite License [1] and the W3C
3-clause BSD License [2]. To contribute to a W3C Test Suite, see the
policies and contribution forms [3].

[1] http://www.w3.org/Consortium/Legal/2008/04-testsuite-license
[2] http://www.w3.org/Consortium/Legal/2008/03-bsd-license
[3] http://www.w3.org/2004/10/27-testcases
*/

//
// Helper Functions for PageVisibility W3C tests
//
var VISIBILITY_STATES =
{
    HIDDEN: "hidden",
    VISIBLE: "visible"
};

var feature_check = false;

//
// All test() functions in the WebPerf PageVis test suite should use pv_test() instead.
//
// pv_test() validates the document.hidden and document.visibilityState attributes
// exist prior to running tests and immediately shows a failure if they do not.
//

function pv_test(func, msg, doc)
{
    if (!doc)
    {
        doc = document;
    }

    // only run the feature check once, unless func == null, in which case,
    // this call is intended as a feature check
    if (!feature_check)
    {
        feature_check = true;

        var hiddenVal = doc.hidden;
        var visStateVal = doc.visibilityState;

        // show a single error that the Page Visibility feature is undefined
        test(function()
        {
            assert_true(hiddenVal !== undefined && hiddenVal != null,
                        "document.hidden is defined and not null.");},
                        "document.hidden is defined and not null.");

        test(function()
        {
            assert_true(visStateVal !== undefined && hiddenVal != null,
                        "document.visibilityState is defined and not null.");},
                        "document.visibilityState is defined and not null.");

    }

    if (func)
    {
        test(func, msg);
    }
}


function test_feature_exists(doc, msg)
{
    if (!msg)
    {
        msg = "";
    }
    var hiddenMsg = "document.hidden is defined" + msg + ".";
    var stateMsg = "document.visibilityState is defined" + msg + ".";
    pv_test(function(){assert_true(document.hidden !== undefined, hiddenMsg);}, hiddenMsg, doc);
    pv_test(function(){assert_true(document.visibilityState !== undefined, stateMsg);}, stateMsg, doc);
}

//
// Common helper functions
//

function test_true(value, msg)
{
    pv_test(function() { assert_true(value, msg); }, msg);
}

function test_equals(value, equals, msg)
{
    pv_test(function() { assert_equals(value, equals, msg); }, msg);
}

//
// asynchronous test helper functions
//

function add_async_result(test_obj, pass_state)
{
    // add assertion to manual test for the pass state
    test_obj.step(function() { assert_true(pass_state) });

    // end manual test
    test_obj.done();
}

function add_async_result_assert(test_obj, func)
{
    // add assertion to manual test for the pass state
    test_obj.step(func);

    // end manual test
    test_obj.done();
}

var open_link;
function TabSwitch()
{
    //var open_link = window.open("http://www.bing.com");
    open_link = window.open('', '_blank');
    step_timeout(function() { open_link.close(); }, 2000);
}
