function ellipseXIntercept(yi, rx, ry)
{
    return rx * Math.sqrt(1 - (yi * yi) / (ry * ry));
}

function scanConvertRoundedRectangleOutside(r, height, lineHeight, corner)
{
    var intervals = [];
    var upperCorner = true;
    var lowerCorner = true;

    if (corner == "upper")
        lowerCorner = false;
    else if (corner == "lower")
        upperCorner = false;

    for (var y = 0; y < height; y += lineHeight) {
        if (y + lineHeight <= r.y || y >= r.y + r.height)
            continue;

        if (upperCorner && (y + lineHeight < r.y + r.ry)) {
            // within the upper rounded corner part of the rectangle
            var dx = ellipseXIntercept(y + lineHeight - r.y - r.ry, r.rx, r.ry);
            intervals.push( { y: y, left: r.x + r.rx - dx, right: r.x + r.width - r.rx + dx} );
        }
        else if (lowerCorner && (y > r.y + r.height - r.ry)) {
            // within the lower rounded corner part of the rectangle
            var dx = ellipseXIntercept(y - (r.y + r.height - r.ry), r.rx, r.ry);
            intervals.push( { y: y, left: r.x + r.rx - dx, right: r.x + r.width - r.rx + dx} );
        }
        else // within the rectangle's vertical edges
            intervals.push( {y: y, left: r.x, right: r.x + r.width} );
    }

    return intervals;
}

function genLeftRoundedRectFloatShapeOutsideRefTest(args)
{
    var leftRoundedRect = args.roundedRect;
    var leftRoundedRectIntervals = scanConvertRoundedRectangleOutside(leftRoundedRect, args.containerHeight, args.lineHeight, args.corner);
    var leftFloatDivs = leftRoundedRectIntervals.map(function(interval) {
        var width = SubPixelLayout.snapToLayoutUnit(interval.right);
        var cls = "left-" + args.floatElementClassSuffix;
        return '<div class="' + cls + '" style="width:' + width + 'px"></div>';
    });
    document.getElementById("left-" + args.insertElementIdSuffix).insertAdjacentHTML('afterend', leftFloatDivs.join("\n"));
    return leftFloatDivs;
}

function getRoundedRectLeftEdge(args)
{
    var leftRoundedRect = args.roundedRect;
    var leftRoundedRectIntervals = scanConvertRoundedRectangleOutside(leftRoundedRect, args.containerHeight, args.lineHeight, args.corner);
    var leftSidePoints = leftRoundedRectIntervals.map(function(interval) {
        var width = SubPixelLayout.snapToLayoutUnit(interval.right);
        return width;
    });
    return leftSidePoints;
}

function genRightRoundedRectFloatShapeOutsideRefTest(args)
{
    var rightRoundedRect = Object.create(args.roundedRect);
    rightRoundedRect.x = args.containerWidth - args.roundedRect.width;
    var rightRoundedRectIntervals = scanConvertRoundedRectangleOutside(rightRoundedRect, args.containerHeight, args.lineHeight, args.corner);
    var rightFloatDivs = rightRoundedRectIntervals.map(function(interval) {
        var width = args.containerWidth - SubPixelLayout.snapToLayoutUnit(interval.left);
        var cls = "right-" + args.floatElementClassSuffix;
        return '<div class="' + cls + '" style="width:' + width + 'px"></div>';
    });
    document.getElementById("right-" + args.insertElementIdSuffix).insertAdjacentHTML('afterend', rightFloatDivs.join("\n"));
}
