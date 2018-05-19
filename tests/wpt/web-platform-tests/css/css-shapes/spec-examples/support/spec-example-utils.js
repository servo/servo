function approxShapeTest(testId, linePrefix, epsilon, lineOffsets) {
    var isPositioned = { 'relative': true, 'fixed': true, 'absolute': true, 'sticky': true },
        testDiv = document.getElementById(testId),
        testOffset = isPositioned[getComputedStyle(testDiv).position] ? 0 : testDiv.offsetLeft,
        firstLine = document.getElementById(linePrefix + '0');

    function runTest() {
        assert_not_equals(firstLine.offsetLeft, testOffset, "Shape layout should have happened already.");

        for (var i = 0; i < lineOffsets.length; i++) {
            var line = document.getElementById(linePrefix + i);
            assert_approx_equals(line.offsetLeft, lineOffsets[i] + testOffset, epsilon, 'Line ' + i + ' is positioned properly');
        }
        done();
    }
    runTest();
}
