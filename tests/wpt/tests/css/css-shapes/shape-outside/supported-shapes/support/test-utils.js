function verifyTextPoints(shape, numLines, tolerance, side) {
    var passed = true;
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
            passed = false;
        }
    }

    return passed;
}
