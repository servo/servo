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

function compareSpaceWithAndWithoutStyle(tag, style, parentStyle, direction) {
    if (!FragmentHelper.isValidChildOfMrow(tag) ||
        FragmentHelper.isEmpty(tag))
        throw `Invalid argument: ${tag}`;

    if (!direction)
      direction = "ltr";
    document.body.insertAdjacentHTML("beforeend", `<div style="position: absolute;">\
<math><mrow dir="${direction}">${MathMLFragments[tag]}</mrow></math>\
<math><mrow dir="${direction}">${MathMLFragments[tag]}</mrow></math>\
</div>`);
    var div = document.body.lastElementChild;

    var styleMath = div.firstElementChild;
    var styleParent = styleMath.firstElementChild;
    if (parentStyle)
        styleParent.setAttribute("style", parentStyle);
    var styleElement = FragmentHelper.element(styleMath);
    styleElement.setAttribute("style", style);
    var styleChild = FragmentHelper.forceNonEmptyElement(styleElement);
    var styleMathBox = styleMath.getBoundingClientRect();
    var styleElementBox = styleElement.getBoundingClientRect();
    var styleChildBox = styleChild.getBoundingClientRect();
    var styleSpace = spaceBetween(styleChildBox, styleMathBox);

    var noStyleMath = div.lastElementChild;
    var noStyleElement = FragmentHelper.element(noStyleMath);
    var noStyleChild = FragmentHelper.forceNonEmptyElement(noStyleElement);
    var noStyleMathBox = noStyleMath.getBoundingClientRect();
    var noStyleElementBox = noStyleElement.getBoundingClientRect();
    var noStyleChildBox = noStyleChild.getBoundingClientRect();
    var noStyleSpace = spaceBetween(noStyleChildBox, noStyleMathBox);

    div.style = "display: none;"; // Hide the div after measurement.

    return {
        left_delta: styleSpace.left - noStyleSpace.left,
        right_delta: styleSpace.right - noStyleSpace.right,
        top_delta: styleSpace.top - noStyleSpace.top,
        bottom_delta: styleSpace.bottom - noStyleSpace.bottom,
        element_width_delta: styleElementBox.width - noStyleElementBox.width,
        element_height_delta: styleElementBox.height - noStyleElementBox.height
    };
}

function compareSizeWithAndWithoutStyle(tag, style) {
    if (!FragmentHelper.isValidChildOfMrow(tag))
        throw `Invalid argument: ${tag}`;

    document.body.insertAdjacentHTML("beforeend", `<div style="position: absolute;">\
<math>${MathMLFragments[tag]}</math>\
<math>${MathMLFragments[tag]}</math>\
</div>`);
    var div = document.body.lastElementChild;

    var styleMath = div.firstElementChild;
    var styleElement = FragmentHelper.element(styleMath);
    styleElement.setAttribute("style", style);
    var styleMathBox = styleMath.getBoundingClientRect();
    var styleElementBox = styleElement.getBoundingClientRect();

    var noStyleMath = div.lastElementChild;
    var noStyleElement = FragmentHelper.element(noStyleMath);
    var noStyleMathBox = noStyleMath.getBoundingClientRect();
    var noStyleElementBox = noStyleElement.getBoundingClientRect();

    div.style = "display: none;"; // Hide the div after measurement.

    return {
        width_delta: styleMathBox.width - noStyleMathBox.width,
        height_delta: styleMathBox.height - noStyleMathBox.height,
        element_width_delta: styleElementBox.width - noStyleElementBox.width,
        element_height_delta: styleElementBox.height - noStyleElementBox.height
    };
};
