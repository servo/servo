function getWritingMode(element, reference) {
    var style = window.getComputedStyle(reference);
    if (style.getPropertyValue("writing-mode") !== "horizontal-tb" ||
        style.getPropertyValue("direction") !== "ltr")
        throw "Reference should have writing mode horizontal-tb and ltr";

    style = window.getComputedStyle(element);
    var param =  {
        rtl: style.getPropertyValue("direction") === "rtl",
        mode: style.getPropertyValue("writing-mode")
    };

    return param;
}

function compareSize(element, reference, epsilon) {
    var param = getWritingMode(element, reference);
    var elementBox = element.getBoundingClientRect();
    var referenceBox = reference.getBoundingClientRect();

    switch(param.mode) {
    case "horizontal-tb":
        assert_approx_equals(elementBox.width, referenceBox.width, epsilon,
                             "inline size");
        assert_approx_equals(elementBox.height, referenceBox.height, epsilon,
                             "block size");
        break;
    case "vertical-lr":
    case "vertical-rl":
        assert_approx_equals(elementBox.width, referenceBox.height, epsilon,
                             "inline size");
        assert_approx_equals(elementBox.height, referenceBox.width, epsilon,
                             "block size");
        break;
    default:
        throw "compareSize: Unrecognized writing-mode value";
    }
}

function compareLayout(element, reference, epsilon) {
    if (element.children.length != reference.children.length)
        throw "Reference should have the same number of children."

    // Compare sizes of elements and children.
    var param = getWritingMode(element, reference);

    compareSize(element, reference, epsilon);
    var elementBox = element.getBoundingClientRect();
    var referenceBox = reference.getBoundingClientRect();
    for (var i = 0; i < element.children.length; i++) {
        var childDisplay = window.
            getComputedStyle(element.children[i]).getPropertyValue("display");
        var referenceChildDisplay = window.
            getComputedStyle(reference.children[i]).getPropertyValue("display");
        if (referenceChildDisplay !== childDisplay)
            throw "compareLayout: children of reference should have the same display values.";
        if (childDisplay === "none")
            continue;

        compareSize(element.children[i], reference.children[i], epsilon);

        var childBox = element.children[i].getBoundingClientRect();
        var referenceChildBox = reference.children[i].getBoundingClientRect();

        switch(param.mode) {
        case "horizontal-tb":
            if (!param.rtl)
                throw "compareLayout: unexpected writing-mode value";
            assert_approx_equals(elementBox.right - childBox.right,
                                 referenceChildBox.left - referenceBox.left,
                                 epsilon,
                                 `inline position (child ${i})`);
            assert_approx_equals(childBox.top - elementBox.top,
                                 referenceChildBox.top - referenceBox.top,
                                 epsilon,
                                 `block position (child ${i})`);
            break;
        case "vertical-lr":
        case "vertical-rl":
            assert_approx_equals(param.rtl ?
                                 elementBox.bottom - childBox.bottom :
                                 childBox.top - elementBox.top,
                                 referenceChildBox.left - referenceBox.left,
                                 epsilon,
                                 `inline position (child ${i})`);
            assert_approx_equals(elementBox.right - childBox.right,
                                 referenceChildBox.top - referenceBox.top,
                                 epsilon,
                                 `block position (child ${i})`);
            break;
        default:
            throw "compareLayout: Unrecognized writing-mode value";
        }
    }
}
