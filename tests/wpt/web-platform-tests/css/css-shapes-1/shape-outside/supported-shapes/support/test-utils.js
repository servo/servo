function verifyTextPoints(shape, numLines, tolerance, side) {
    var failed = false;
    if (tolerance === undefined)
        tolerance = 0.5;
    if (side === undefined)
        side = "left";

    if (side === "right")
        shape.roundedRect.x = shape.containerWidth - (shape.roundedRect.x + shape.roundedRect.width);

    var expected = getRoundedRectLeftEdge(shape);

    for(var i = 0; i < numLines; i++) {
       var line = document.getElementById('test'+i);
       var actual = line.getBoundingClientRect().left;
       if (side === "right")
            actual = shape.containerWidth - (actual + line.getBoundingClientRect().width);

        if( Math.abs( (actual - expected[i])) > tolerance ){
            line.style.setProperty('color', 'red');
            console.log('diff: ' + Math.abs(actual - expected[i]));
            failed = true;
        }
    }
    if (window.done) {
        assert_false(failed, "Lines positioned properly around the shape.");
        done();
    }
}
