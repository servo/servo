function spaceBetween(childBox, parentBox) {
    return {
        left: childBox.left - parentBox.left,
        right: parentBox.right - childBox.right,
        top: childBox.top - parentBox.top,
        bottom: parentBox.bottom - childBox.bottom
    };
}

function measureSpaceAround(id) {
    var mrow = document.getElementById(id);
    var mrowBox = mrow.getBoundingClientRect();
    var parentBox = mrow.parentNode.getBoundingClientRect();
    var childBox = mrow.firstElementChild.getBoundingClientRect();
    return spaceBetween(childBox, parentBox);
}

function compareSpaceWithAndWithoutStyle(tag, style) {
    if (!FragmentHelper.isValidChildOfMrow(tag) ||
        FragmentHelper.isEmpty(tag))
        throw `Invalid argument: ${tag}`;

    document.body.insertAdjacentHTML("beforeend", `<div>\
<math>${MathMLFragments[tag]}</math>\
<math>${MathMLFragments[tag]}</math>\
</div>`);
    var div = document.body.lastElementChild;

    var styleMath = div.firstElementChild;
    var styleElement = FragmentHelper.element(styleMath);
    styleElement.setAttribute("style", style);
    var styleChild = FragmentHelper.forceNonEmptyElement(styleElement);
    var styleBox = styleMath.getBoundingClientRect();
    var styleChildBox = styleChild.getBoundingClientRect();
    var styleSpace = spaceBetween(styleChildBox, styleBox);

    var noStyleMath = div.lastElementChild;
    var noStyleElement = FragmentHelper.element(noStyleMath);
    var noStyleChild = FragmentHelper.forceNonEmptyElement(noStyleElement);
    var noStyleBox = noStyleMath.getBoundingClientRect();
    var noStyleChildBox = noStyleChild.getBoundingClientRect();
    var noStyleSpace = spaceBetween(noStyleChildBox, noStyleBox);

    div.style = "display: none;"; // Hide the div after measurement.

    return {
        left_delta: styleSpace.left - noStyleSpace.left,
        right_delta: styleSpace.right - noStyleSpace.right,
        top_delta: styleSpace.top - noStyleSpace.top,
        bottom_delta: styleSpace.bottom - noStyleSpace.bottom
    };
}

function compareSizeWithAndWithoutStyle(tag, style) {
    if (!FragmentHelper.isValidChildOfMrow(tag))
        throw `Invalid argument: ${tag}`;

    document.body.insertAdjacentHTML("beforeend", `<div>\
<math>${MathMLFragments[tag]}</math>\
<math>${MathMLFragments[tag]}</math>\
</div>`);
    var div = document.body.lastElementChild;

    var styleMath = div.firstElementChild;
    var styleElement = FragmentHelper.element(styleMath);
    styleElement.setAttribute("style", style);
    var styleBox = styleMath.getBoundingClientRect();

    var noStyleMath = div.lastElementChild;
    var noStyleElement = FragmentHelper.element(noStyleMath);
    var noStyleBox = noStyleMath.getBoundingClientRect();

    div.style = "display: none;"; // Hide the div after measurement.

    return {
        width_delta: styleBox.width - noStyleBox.width,
        height_delta: styleBox.height - noStyleBox.height
    };
};
