// META: title=FormData: changes to entry list during iteration

// These are tests for next()'s behavior as specified in
// https://webidl.spec.whatwg.org/#es-iterator-prototype-object

"use strict";

function createFormData(input) {
    const formData = new FormData();

    for (const [name, value] of input) {
        formData.append(name, value);
    }

    return formData;
}

test(() => {
    const formData = createFormData([["foo", "0"],
                                     ["baz", "1"],
                                     ["BAR", "2"]]);
    const actualKeys = [];
    const actualValues = [];
    for (const [name, value] of formData) {
        actualKeys.push(name);
        actualValues.push(value);
        formData.delete("baz");
    }
    assert_array_equals(actualKeys, ["foo", "BAR"]);
    assert_array_equals(actualValues, ["0", "2"]);
}, "Iteration skips elements removed while iterating");

test(() => {
    const formData = createFormData([["foo", "0"],
                                     ["baz", "1"],
                                     ["BAR", "2"],
                                     ["quux", "3"]]);
    const actualKeys = [];
    const actualValues = [];
    for (const [name, value] of formData) {
        actualKeys.push(name);
        actualValues.push(value);
        if (name === "baz")
            formData.delete("foo");
    }
    assert_array_equals(actualKeys, ["foo", "baz", "quux"]);
    assert_array_equals(actualValues, ["0", "1", "3"]);
}, "Removing elements already iterated over causes an element to be skipped during iteration");

test(() => {
    const formData = createFormData([["foo", "0"],
                                     ["baz", "1"],
                                     ["BAR", "2"],
                                     ["quux", "3"]]);
    const actualKeys = [];
    const actualValues = [];
    for (const [name, value] of formData) {
        actualKeys.push(name);
        actualValues.push(value);
        if (name === "baz")
            formData.append("X-yZ", "4");
    }
    assert_array_equals(actualKeys, ["foo", "baz", "BAR", "quux", "X-yZ"]);
    assert_array_equals(actualValues, ["0", "1", "2", "3", "4"]);
}, "Appending a value pair during iteration causes it to be reached during iteration");
