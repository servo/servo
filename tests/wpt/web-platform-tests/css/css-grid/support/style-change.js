function evaluateStyleChange(element, phase, expectedProperty, expectedResult) {
    element.className += " " + phase;
    element.setAttribute(expectedProperty, expectedResult);
    checkLayout("." + phase);
}
