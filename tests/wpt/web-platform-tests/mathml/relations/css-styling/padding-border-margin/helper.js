function measureSpaceAround(id) {
    var mrow = document.getElementById(id);
    var mrowBox = mrow.getBoundingClientRect();
    var parentBox = mrow.parentNode.getBoundingClientRect();
    var childBox = mrow.firstElementChild.getBoundingClientRect();
    return {
        left: childBox.left - parentBox.left,
        right: parentBox.right - childBox.right,
        top: childBox.top - parentBox.top,
        bottom: parentBox.bottom - childBox.bottom
    };
}
