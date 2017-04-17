function approxShapeTest(testId, linePrefix, epsilon, lineOffsets) {
    var isPositioned = { 'relative': true, 'fixed': true, 'absolute': true, 'sticky': true },
        loops = 0,
        testDiv = document.getElementById(testId),
        testOffset = isPositioned[getComputedStyle(testDiv).position] ? 0 : testDiv.offsetLeft,
        firstLine = document.getElementById(linePrefix + '0');

    function runTest() {
        if (firstLine.offsetLeft == testOffset) {
            // wait for the shape image to load and layout to happen
            if (loops > 100)
                assert_unreached("Giving up waiting for shape layout to happen!");
            else
                loops++;
            window.setTimeout(runTest, 5);
            return;
        }

        for (var i = 0; i < lineOffsets.length; i++) {
            var line = document.getElementById(linePrefix + i);
            assert_approx_equals(line.offsetLeft, lineOffsets[i] + testOffset, epsilon, 'Line ' + i + ' is positioned properly');
        }
        done();
    }
    runTest();
}
