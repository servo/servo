"use strict";
// TODO: Test interaction with SVG, specificity of SVG transform attribute,
// inlines
//
// TODO: Test more CSSOM stuff
//
// TODO: Break into multiple files?
//
// Not for now: transitions, animations
//@{
var div = document.querySelector("#test");
var divWidth = 100, divHeight = 50;
var divParentWidth = 120, divParentHeight = 70;
// Arbitrarily chosen epsilon that makes browsers mostly pass with some extra
// breathing room, since the specs don't define rounding for display.
//
// FIXME: These need to be specified.
// https://www.w3.org/Bugs/Public/show_bug.cgi?id=15709
var pixelEpsilon = 1.5;
// A much smaller epsilon for computed style values, since there's no good
// reason for those to be very far off.  Some UAs do a bunch of rounding, but
// it should still be good to one decimal place.
var computedEpsilon = 0.05;

// percentagesAndLengths and lengths are both ordered with the most interesting
// things first, so you can truncate them to avoid undue combinatorial
// explosion.
var percentagesAndLengths = [
	".0", "-1px", "1pt", "53.7px", "-50%",
	"1em", "1px", "0.12%", "0.12px", "0%",
	"-53.7px", "0.0px",
	"1ex", "1in", "1cm", "1mm", "1pc"];
var lengths = percentagesAndLengths.filter(function(s){return !/%$/.test(s)});

var rotateAngles = [
	"-7deg", "0", "0deg", "22.5deg", "45deg", "86.451deg", "90deg", "180deg",
	"270deg", "452deg",
	"-1rad", "0rad", "1rad", "6.28rad",
	"0.721turn", "256grad"];
// Do not test values close to 90 degrees, because this will cause coordinates
// to get large.  The maximum values for coordinates are (of course) not
// defined, and even if they were, the result would be extremely sensitive to
// rounding error.
var skewAngles = [
	"-80deg", "0", "-45deg", "-32.6deg", "-0.05deg", "0deg", "0.05deg", "32.6deg",
	"45deg", "80deg", "300deg",
	"-0.3rad", "0rad", "0.3rad", "2.9rad",
	"0.921turn", "22grad"];

var emPixels = parseFloat(getComputedStyle(div).fontSize);
div.style.fontSize = "1ex";
var exPixels = parseFloat(getComputedStyle(div).fontSize);
div.removeAttribute("style");

var switchStyles = document.querySelectorAll("style.switch");
[].forEach.call(switchStyles, function(style) { style.disabled = true });
//@}

// Track how many tests we're running for each section of the test files
//@{
var section;
var sectionCounts = {};
add_result_callback(function() {
	if (!(section in sectionCounts)) {
		sectionCounts[section] = 0;
	}
	sectionCounts[section]++;
});
add_completion_callback(function() {
	var msg = "Tests: ";
	var total = 0;
	for (var key in sectionCounts) {
		msg += key + " " + sectionCounts[key] + ", ";
		total += sectionCounts[key];
	}
	msg += "total " + total;
	document.body.appendChild(document.createTextNode(msg));
});
//@}

/**
 * Account for prefixing so that I can check whether browsers actually follow
 * the spec.  Obviously, in any final version of the test, only the unprefixed
 * property will be tested.  Usage: prefixProp("transformOrigin") ==
 * "msTransformOrigin", "mozTransformOrigin", etc. as appropriate.
 */
function prefixProp(s) {
//@{
	if (s in div.style) {
		return s;
	}
	s = s[0].toUpperCase() + s.slice(1);
	var prefixes = ["ms", "Moz", "moz", "webkit", "O"];
	for (var i = 0; i < prefixes.length; i++) {
		if ((prefixes[i] + s) in div.style) {
			return prefixes[i] + s;
		}
	}
	return s;
}
//@}

/**
 * Likewise, but gives the hyphenated version.
 * prefixHyphenatedProp("transform-origin") is "-ms-transform-origin",
 * "-moz-transform-origin", etc.
 */
function prefixHyphenatedProp(s) {
//@{
	s = s.split("-")
		.map(function(bit, i) {
			return i == 0 ? bit : bit[0].toUpperCase() + bit.slice(1)
		})
		.join("");
	var prefixed = prefixProp(s);
	if (prefixed != s) {
		// Capitalize the first letter so the return line adds a leading
		// hyphen, e.g., "msTransform" -> "MsTransform" -> "-ms-transform"
		// instead of "ms-transform".
		s = prefixed[0].toUpperCase() + prefixed.slice(1);
	}
	return s.replace(/([A-Z])/g, "-$1")
		.toLowerCase();
}
//@}

/**
 * Accepts a string that's a CSS length or percentage, and returns a number of
 * pixels (not a string), or null if parsing fails.  For percentages to be
 * accepted, percentRef must not be undefined.
 */
function convertToPx(input, percentRef) {
//@{
	var match = /^([-+]?[0-9]+|[-+]?[0-9]*\.[0-9]+)(em|ex|in|cm|mm|pt|pc|px|%)?$/.exec(input);
	if (!match) {
		return null;
	}
	var amount = Number(match[1]);
	var unit = match[2];
	if (amount == 0) {
		return 0;
	}
	if (!unit) {
		return null;
	}
	if (unit == "%" && percentRef === undefined) {
		return null;
	}
	return amount * {
		em: emPixels,
		ex: exPixels,
		in: 72/0.75,
		cm: (1/2.54)*72/0.75,
		mm: (1/25.4)*72/0.75,
		pt: 1/0.75,
		pc: 12/0.75,
		px: 1,
		"%": percentRef/100,
	}[unit];
}
//@}

/**
 * Accepts a string that's a CSS angle, and returns a number of radians (not a
 * string), or null if parsing fails.  0 is accepted without a unit, as the
 * transform spec requires.
 */
function convertToRad(input) {
//@{
	var match = /^([-+]?[0-9]+|[-+]?[0-9]*\.[0-9]+)(deg|grad|rad|turn|)$/.exec(input);
	if (!match) {
		return null;
	}
	var amount = Number(match[1]);
	var unit = match[2];
	if (unit == "" && amount == 0) {
		return 0;
	}
	if (unit == "") {
		return null;
	}
	return amount * {
		deg: Math.PI/180,
		grad: Math.PI/200,
		rad: 1,
		turn: 2*Math.PI,
	}[unit];
}
//@}

/**
 * Multiplies two or more 2x3 matrices passed as one-dimensional column-major
 * arrays (interpreted as 3x3 matrices with bottom row 0 0 1).
 */
function mxmul23(A, B) {
//@{
	if (arguments.length > 2) {
		return mxmul23(A, mxmul23.apply(this, [].slice.call(arguments, 1)));
	}
	return [
		A[0]*B[0] + A[2]*B[1],
		A[1]*B[0] + A[3]*B[1],
		A[0]*B[2] + A[2]*B[3],
		A[1]*B[2] + A[3]*B[3],
		A[0]*B[4] + A[2]*B[5] + A[4],
		A[1]*B[4] + A[3]*B[5] + A[5]
	];
}
//@}

/**
 * Multiplies two or more 4x4 matrices passed as one-dimensional column-major
 * arrays.
 */
function mxmul44(A, B) {
//@{
	if (arguments.length > 2) {
		return mxmul44(A, mxmul44.apply(this, [].slice.call(arguments, 1)));
	}
	A = [A.slice(0, 4), A.slice(4, 8), A.slice(8, 12), A.slice(12, 16)];
	B = [B.slice(0, 4), B.slice(4, 8), B.slice(8, 12), B.slice(12, 16)];
	var C = [];
	for (var i = 0; i < 4; i++) {
		C.push([]);
		for (var j = 0; j < 4; j++) {
			C[i].push(0);
			for (var k = 0; k < 4; k++) {
				C[i][j] += B[i][k]*A[k][j];
			}
		}
	}
	return C[0].concat(C[1]).concat(C[2]).concat(C[3]);
}
//@}

/**
 * Given a sixteen-element numeric array mx in column-major order, returns true
 * if it's equivalent to a six-element array (a 2D matrix), false otherwise.
 */
function is2dMatrix(mx) {
//@{
	// Use a really small epsilon here.  Otherwise we'll think perspective
	// matrices are 2D.
	var e = 1.0e-5;
	return Math.abs(mx[2]) < e
		&& Math.abs(mx[3]) < e

		&& Math.abs(mx[6]) < e
		&& Math.abs(mx[7]) < e

		&& Math.abs(mx[8]) < e
		&& Math.abs(mx[9]) < e
		&& Math.abs(mx[10] - 1) < e
		&& Math.abs(mx[11]) < e

		&& Math.abs(mx[14]) < e
		&& Math.abs(mx[15] - 1) < e;
}
//@}

/**
 * Returns the rotation matrix used for rotate3d(x, y, z, angle).
 */
function getRotationMatrix(x, y, z, angle) {
//@{
	var rads = convertToRad(angle);
	var len = Math.sqrt(x*x + y*y + z*z);
	x /= len;
	y /= len;
	z /= len;
	var sc = Math.sin(rads/2)*Math.cos(rads/2);
	var sq = Math.sin(rads/2)*Math.sin(rads/2);
	var ret =
		[1 - 2*(y*y + z*z)*sq,
		2*(x*y*sq + z*sc),
		2*(x*z*sq - y*sc),
		0,

		2*(x*y*sq - z*sc),
		1 - 2*(x*x + z*z)*sq,
		2*(y*z*sq + x*sc),
		0,

		2*(x*z*sq + y*sc),
		2*(y*z*sq - x*sc),
		1 - 2*(x*x + y*y)*sq,
		0,

		0, 0, 0, 1];
	return ret;
}
//@}

/**
 * Sets the styles of the test div, its parent, its grandparent, and the body.
 * It will sometimes use CSSOM and sometimes setAttribute(), in an arbitrary
 * but deterministic fashion.  Each of the four arguments can be either
 * undefined (meaning not to touch that element's style), or an object.  The
 * object has a format like
 *   {transform: "scale(2)", transformOrigin: "top 10px"}.
 */
function setStyles(divStyle, parentStyle, grandparentStyle, bodyStyle) {
//@{
	// If any existing styles are being overwritten, toggle useCssom.  TODO:
	// This logic seems wrong, although it doesn't matter much.
	if ((setStyles.currentStyles[0] && divStyle && Object.keys(divStyle).length)
	|| (setStyles.currentStyles[1] && parentStyle && Object.keys(parentStyle).length)
	|| (setStyles.currentStyles[2] && grandparentStyle && Object.keys(grandparentStyle).length)
	|| (setStyles.currentStyles[3] && bodyStyle && Object.keys(bodyStyle).length)) {
		setStyles.useCssomCounter++;
		setStyles.useCssomCounter %= 17;
		setStyles.useCssom = Boolean(setStyles.useCssomCounter % 2);
	}

	if (divStyle) {
		setStyles.currentStyles[0] = setStyle(div, divStyle);
	}
	if (parentStyle) {
		setStyles.currentStyles[1] = setStyle(div.parentNode, parentStyle);
	}
	if (grandparentStyle) {
		setStyles.currentStyles[2] =
			setStyle(div.parentNode.parentNode, grandparentStyle);
	}
	if (bodyStyle) {
		setStyles.currentStyles[3] =
			setStyle(document.body, bodyStyle);
	}
}
//@}
setStyles.currentStyles = ["", "", "", ""];
setStyles.useCssomCounter = 0;
setStyles.useCssom = false;

/**
 * Helper function for setStyle().
 */
function setStyle(node, style) {
//@{
	// TODO: The setAttribute() here is redundant in theory, but sometimes
	// removeAttribute() randomly fails in WebKit.  We should remove the
	// redundant line and let WebKit fail -- there's no reason to work around
	// its bugs.  But that interferes with checking test correctness, so leave
	// it for now.
	node.setAttribute("style", "");
	node.removeAttribute("style");

	var ret = [];
	// Used if !setStyles.useCssom
	var textToSet = [];
	for (var prop in style) {
		if (style[prop] == "") {
			continue;
		}
		var hyphenatedProp = prop.replace(/([A-Z])/g, "-$1").toLowerCase();
		ret.push(hyphenatedProp + ": " + style[prop]);

		if (setStyles.useCssom) {
			node.style[prefixProp(prop)] = style[prop];
		} else {
			textToSet.push(prefixHyphenatedProp(hyphenatedProp)
				+ ": " + style[prop]);
		}
	}
	if (!setStyles.useCssom) {
		node.setAttribute("style", textToSet.join("; "));
	}
	return ret.join("; ");
}
//@}

/**
 * Returns a string describing the style attributes currently in effect on the
 * test div and its parents/grandparents, like
 *   with "transform: scale(1.2, 1); transform-origin: 50% 50%", set via CSSOM
 * or
 *   with "transform: matrix(4, -7, 2.3, -3.8, 6, 6)" on test div's
 *   grandparent, "transform: matrix(4, -7, 2.3, -3.8, 6, 6)" on its parent,
 *   "transform: none; transform-origin: 50% 50%" on test div, set via
 *   setAttribute()
 * This relies on setStyles() being used, rather than direct manipulation of
 * attributes or CSSOM.
 */
function getStyleDescription() {
//@{
	var styleText = [];
	if (setStyles.currentStyles[3]) {
		styleText.push('"' + setStyles.currentStyles[3] + '"'
			+ " on body");
	}
	if (setStyles.currentStyles[2]) {
		styleText.push('"' + setStyles.currentStyles[2] + '"'
			+ " on test div's grandparent");
	}
	if (setStyles.currentStyles[1]) {
		styleText.push('"' + setStyles.currentStyles[1] + '"'
			+ " on test div's parent");
	}
	if (setStyles.currentStyles[0]) {
		styleText.push('"' + setStyles.currentStyles[0] + '"'
			+ (styleText.length ? " on test div" : ""));
	}
	if (styleText.length) {
		return "with "
			+ styleText.join(", ")
			+ ", set via " + (setStyles.useCssom ? "CSSOM" : "setAttribute()");
	}
	return "with no attributes set";
}
//@}


/**
 * Tests that style="transform: value" results in transformation by the matrix
 * mx, which may have either six or sixteen entries.  Checks both the computed
 * value and bounding box.
 */
function testTransform(value, mx) {
//@{
	setStyles({transform: value});
	test(function() {
		testComputedTransform(mx);
	}, "getComputedStyle(div).transform " + getStyleDescription());
	testTransformedBoundary(value, mx);
}
//@}

/**
 * Tests that div's computed style for transform is "matrix(...)" or
 * "matrix3d(...)", as appropriate.  mx can have either zero, six, or sixteen
 * entries.
 *
 * If mx has zero entries, that means the transform is supposed to parse the
 * same as "none" or be a parse error.
 *
 * FIXME: This now contradicts the spec, after
 * https://www.w3.org/Bugs/Public/show_bug.cgi?id=15797 was fixed.
 *
 * If mx has six entries, it's equivalent to a 4x4 matrix with 0's and 1's in
 * the right places.  If it has sixteen entries, the required output format is
 * still matrix() instead of matrix3d() if it's equivalent to a 2D matrix.
 */
function testComputedTransform(mx) {
//@{
	if (mx.length == 0) {
		assert_equals(getComputedStyle(div)[prefixProp("transform")], "none",
			"wrong computed value");
		return;
	}
	if (mx.length == 6) {
		mx = [mx[0], mx[1], 0, 0,  mx[2], mx[3], 0, 0,  0, 0, 1, 0,  mx[4], mx[5], 0, 1];
	}
	var computed = getComputedStyle(div)[prefixProp("transform")];
	if (is2dMatrix(mx)) {
		var re = /^matrix\(([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+)\)$/;
		assert_regexp_match(computed, re, "computed value has unexpected form for 2D matrix");
		var msg = ' (actual: "' + computed + '"; '
			+ 'expected: "matrix(' + [mx[0], mx[1], mx[4], mx[5], mx[12], mx[13]].join(', ') +')")';
		var match = re.exec(computed);
		assert_approx_equals(Number(match[1]), mx[0], computedEpsilon,
			"getComputedStyle matrix component 0" + msg);
		assert_approx_equals(Number(match[2]), mx[1], computedEpsilon,
			"getComputedStyle matrix component 1" + msg);
		assert_approx_equals(Number(match[3]), mx[4], computedEpsilon,
			"getComputedStyle matrix component 2" + msg);
		assert_approx_equals(Number(match[4]), mx[5], computedEpsilon,
			"getComputedStyle matrix component 3" + msg);
		assert_approx_equals(Number(match[5]), mx[12], computedEpsilon,
			"getComputedStyle matrix component 4" + msg);
		assert_approx_equals(Number(match[6]), mx[13], computedEpsilon,
			"getComputedStyle matrix component 5" + msg);
		return;
	}

	var re = /^matrix3d\(([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+), ([^,]+?)\)$/;
	assert_regexp_match(computed, re, "computed value has unexpected form for 3D matrix");
	var msg = ' (actual: "' + computed + '"; '
		+ 'expected: "matrix3d(' + mx.join(', ') +')")';
	var match = re.exec(computed);
	for (var i = 0; i < 16; i++) {
		assert_approx_equals(Number(match[i + 1]), mx[i], computedEpsilon,
			"getComputedStyle matrix component " + i + msg);
	}
}
//@}

/**
 * Tests that div.style.transform and div.style.cssText are set as would be
 * appropriate if the div has style="transform: value".  FIXME: This is not
 * specified anywhere yet.
 * https://www.w3.org/Bugs/Public/show_bug.cgi?id=15710
 */
function testInlineTransform(expectedValue) {
//@{
	assert_equals(div.style[prefixProp("transform")], expectedValue,
		"div.style.transform");
	if (expectedValue == "") {
		assert_equals(div.style.cssText, "", "div.style.cssText");
	} else {
		// Browsers vary in whether they output ";" or "; " at the end
		assert_equals(div.style.cssText.replace(/;? ?$/, ""),
			prefixHyphenatedProp("transform") + ": " + expectedValue,
			"div.style.cssText");
	}
}
//@}

/**
 * Tests that
 *   style="transform: transformValue; transform-origin: transformOriginValue"
 * results in the boundary box that you'd get from transforming with a matrix
 * of mx around an offset of [xOffset, yOffset].  transformOriginValue defaults
 * to nothing (thus the default of "50% 50%"), xOffset to divWidth/2, yOffset
 * to divHeight/2, zOffset to 0.
 *
 * transformValue can also be an array of three values.  If it is, they're used
 * for the test div's grandparent, its parent, and the test div itself,
 * respectively.  mx should then be the entries of the matrix of all three
 * transforms multiplied together.
 *
 * mx can have zero, six, or sixteen entries.  If it has zero, it's the same as
 * the identity matrix.
 */
function testTransformedBoundary(transformValue, mx,
                                 transformOriginValue, xOffset, yOffset, zOffset) {
//@{
	if (mx.length == 0) {
		mx = [1, 0, 0, 0,  0, 1, 0, 0,  0, 0, 1, 0,  0, 0, 0, 1];
	}
	if (mx.length == 6) {
		mx = [mx[0], mx[1], 0, 0,  mx[2], mx[3], 0, 0,  0, 0, 1, 0,  mx[4], mx[5], 0, 1];
	}

	// Don't test singular matrices for now.  IE fails some of them, which
	// might be due to getBoundingClientRect() instead of transforms.  Only
	// skipped for 2D matrices, for sanity's sake (don't want to compute 4x4
	// determinants).
	if (is2dMatrix(mx)
	&& mx[0]*mx[5] - mx[1]*mx[4] === 0) {
		return;
	}

	if (transformOriginValue === undefined) {
		transformOriginValue = "";
	}
	if (xOffset === undefined) {
		xOffset = divWidth/2;
	}
	if (yOffset === undefined) {
		yOffset = divHeight/2;
	}
	if (zOffset === undefined) {
		zOffset = 0;
	}

	// Compute the expected bounding box by applying the given matrix to the
	// vertices of the test div's border box.
	var originalPoints = [[0, 0], [0, divHeight], [divWidth, 0], [divWidth, divHeight]];
	var newPoints = [];
	for (var i = 0; i < originalPoints.length; i++) {
		var x = originalPoints[i][0] - xOffset;
		var y = originalPoints[i][1] - yOffset;
		var z = -zOffset;
		// Perspective; hope w isn't 0.  FIXME: Precise behavior isn't really
		// defined anywhere, although the intent is relatively clear as long as
		// nothing gets w <= 0:
		// https://www.w3.org/Bugs/Public/show_bug.cgi?id=15605
		var newW = mx[3]*x + mx[7]*y + mx[11]*z + mx[15];
		if (newW <= 0) {
			// There's no reasonable bounding rect -- theoretically the box is
			// probably infinite or invisible.
			return;
		}
		var newX = (mx[0]*x + mx[4]*y + mx[8]*z + mx[12])/newW + xOffset;
		var newY = (mx[1]*x + mx[5]*y + mx[9]*z + mx[13])/newW + yOffset;
		newPoints.push([newX, newY]);
	}

	// Pick a different <style class=switch> for each test; they shouldn't
	// affect results, so it's fine to just alternate.  We cycle through using
	// a reasonably large prime number (19) so that when the tests are
	// repetitive, we're unlikely to keep hitting the same styles for the same
	// sort of test.
	if (testTransformedBoundary.switchStyleIdx === undefined) {
		testTransformedBoundary.switchStyleIdx = switchStyles.length - 1;
	}
	switchStyles[testTransformedBoundary.switchStyleIdx % switchStyles.length].disabled = true;
	testTransformedBoundary.switchStyleIdx++;
	testTransformedBoundary.switchStyleIdx %= 19;
	switchStyles[testTransformedBoundary.switchStyleIdx % switchStyles.length].disabled = false;

	if (typeof transformValue == "string") {
		setStyles({transform: transformValue,
			transformOrigin: transformOriginValue})
	} else {
		setStyles({transform: transformValue[2],
			transformOrigin: transformOriginValue},
			{transform: transformValue[1]}, {transform: transformValue[0]});
	}

	test(function() {
		// FIXME: We assume getBoundingClientRect() returns the rectangle
		// that contains the transformed box, not the untransformed box.
		// This is not actually specified anywhere.  Likewise the other CSSOM
		// functions we use here.
		// https://www.w3.org/Bugs/Public/show_bug.cgi?id=15430
		//
		// First we do getBoundingClientRect().
		var expectedTop = Math.min.apply(null, newPoints.map(function(p){return p[1]}));
		var expectedRight = Math.max.apply(null, newPoints.map(function(p){return p[0]}));
		var expectedBottom = Math.max.apply(null, newPoints.map(function(p){return p[1]}));
		var expectedLeft = Math.min.apply(null, newPoints.map(function(p){return p[0]}));
		var rect = div.getBoundingClientRect();
		var msg = " (actual " + rect.top.toFixed(3) + ", "
			+ rect.right.toFixed(3) + ", "
			+ rect.bottom.toFixed(3) + ", "
			+ rect.left.toFixed(3) + "; "
			+ "expected " + expectedTop.toFixed(3) + ", "
			+ expectedRight.toFixed(3) + ", "
			+ expectedBottom.toFixed(3) + ", "
			+ expectedLeft.toFixed(3) + ")";
		assert_approx_equals(rect.top, expectedTop, pixelEpsilon, "top" + msg);
		assert_approx_equals(rect.right, expectedRight, pixelEpsilon, "right" + msg);
		assert_approx_equals(rect.bottom, expectedBottom, pixelEpsilon, "bottom" + msg);
		assert_approx_equals(rect.left, expectedLeft, pixelEpsilon, "left" + msg);
		assert_approx_equals(rect.width, expectedRight - expectedLeft, pixelEpsilon, "width" + msg);
		assert_approx_equals(rect.height, expectedBottom - expectedTop, pixelEpsilon, "height" + msg);
	}, "getBoundingClientRect() " + getStyleDescription() + "; "
	+ "switch style " + (testTransformedBoundary.switchStyleIdx % switchStyles.length));

	// Now elementFromPoint.  Don't test for points inside if two opposite
	// sides are separated by less than (arbitrarily chosen) 5px.  Take the
	// left side and project the center right point onto it, and the top side
	// with the center bottom point projected onto it.  Also don't assume that
	// anything is inside the element if we have transform-style: preserve-3d
	// anywhere, because the test div might be covered by something else.
	var getVectorToLine = function(origin, point, line) {
		point = [point[0] - origin[0], point[1] - origin[1]];
		line = [line[0] - origin[0], line[1] - origin[1]];
		var lineLength = Math.sqrt(line[0]*line[0] + line[1]*line[1]);
		line[0] /= lineLength;
		line[1] /= lineLength;
		return [
			point[0] - line[0] * (point[0]*line[0] + point[1]*line[1]),
			point[1] - line[1] * (point[0]*line[0] + point[1]*line[1])
		];
	};
	var getOrthogonalDistance = function(origin, point, line) {
		var projected = getVectorToLine(origin, point, line);
		return Math.sqrt(projected[0]*projected[0] + projected[1]*projected[1]);
	};
	var distance1 = getOrthogonalDistance(newPoints[0],
		[(newPoints[2][0] + newPoints[3][0])/2, (newPoints[2][1] + newPoints[3][1])/2],
		newPoints[1]
	);
	var distance2 = getOrthogonalDistance(newPoints[0],
		[(newPoints[1][0] + newPoints[3][0])/2, (newPoints[1][1] + newPoints[3][1])/2],
		newPoints[2]
	);

	var testInside = distance1 >= 5 && distance2 >= 5
		&& getComputedStyle(div.parentNode)[prefixProp("transformStyle")] == "flat"
		&& getComputedStyle(div.parentNode.parentNode)[prefixProp("transformStyle")] == "flat";

	test(function() {
		if (testInside) {
			// We'll test inside points as well as outside.  Take weighted
			// averages of the corners to get points that are 1/4 of the way
			// inside.
			var weightPoints = function(topLeft, bottomLeft, topRight, bottomRight) {
				var sum = topLeft + bottomLeft + topRight + bottomRight;
				topLeft /= sum;
				bottomLeft /= sum;
				topRight /= sum;
				bottomRight /= sum;
				return [
					newPoints[0][0]*topLeft + newPoints[1][0]*bottomLeft
					+ newPoints[2][0]*topRight + newPoints[3][0]*bottomRight,
					newPoints[0][1]*topLeft + newPoints[1][1]*bottomLeft
					+ newPoints[2][1]*topRight + newPoints[3][1]*bottomRight
				];
			};

			var pointsInside = [
				weightPoints(3, 3, 1, 1),
				weightPoints(1, 1, 3, 3),
				weightPoints(3, 1, 3, 1),
				weightPoints(1, 3, 1, 3),
			];

			pointsInside.forEach(function(p) {
				document.documentElement.style.position = "relative";
				document.documentElement.style.left = (10 - p[0]) + "px";
				document.documentElement.style.top = (10 - p[1]) + "px";
				assert_equals(document.elementFromPoint(10, 10), div,
					"Point (" + p[0] + ", " + p[1] + ") is not in the test div.  "
					+ "Expected vertices: " + newPoints.map(function(p) {
						return "(" + p[0] + ", " + p[1] + ")";
					}).join("; ")
				);
			});
		}

		// Test outside points regardless.  Make them always 5px outside, so
		// it should be valid even if the test div is very skinny.
		var getOutsideEdge = function(origin, point1, point2, line) {
			var point = [(point1[0] + point2[0])/2, (point1[1] + point2[1])/2];
			var projected = getVectorToLine(origin, point, line);
			var len = Math.sqrt(projected[0]*projected[0] + projected[1]*projected[1]);
			projected[0] *= -5/len;
			projected[1] *= -5/len;
			projected[0] += origin[0];
			projected[1] += origin[1];
			return projected;
		};
		var pointsOutside = [
			getOutsideEdge(newPoints[0], newPoints[1], newPoints[3], newPoints[2]),
			getOutsideEdge(newPoints[2], newPoints[0], newPoints[1], newPoints[3]),
			getOutsideEdge(newPoints[1], newPoints[0], newPoints[2], newPoints[3]),
			getOutsideEdge(newPoints[0], newPoints[2], newPoints[3], newPoints[1]),
		];

		pointsOutside.forEach(function(p) {
			document.documentElement.style.position = "relative";
			document.documentElement.style.left = (10 - p[0]) + "px";
			document.documentElement.style.top = (10 - p[1]) + "px";
			assert_not_equals(document.elementFromPoint(10, 10), div,
				"Point (" + p[0] + ", " + p[1] + ") is in the test div.  "
				+ "Expected vertices: " + newPoints.map(function(p) {
					return "(" + p[0] + ", " + p[1] + ")";
				}).join("; ")
			);
		});
	}, "elementFromPoint() " + (testInside ? "" : "(outside points only) ")
	+ getStyleDescription() + "; "
	+ "switch style " + (testTransformedBoundary.switchStyleIdx % switchStyles.length));

	if (typeof transformValue != "string") {
		setStyles(undefined, {}, {});
	}

	document.documentElement.setAttribute("style", "");
	document.documentElement.removeAttribute("style");
}
//@}

/**
 * Test that "transform-origin: value" acts like the origin is at
 * (expectedHoriz, expectedVert), where the latter two parameters can be
 * keywords, percentages, or lengths.  Tests both that the computed value is
 * correct, and that the boundary box is as expected for a 45-degree rotation.
 */
function testTransformOrigin(value, expectedX, expectedY, expectedZ) {
//@{
	if (expectedX == "left") {
		expectedX = "0%";
	} else if (expectedX == "center") {
		expectedX = "50%";
	} else if (expectedX == "right") {
		expectedX = "100%";
	}
	if (expectedY == "top") {
		expectedY = "0%";
	} else if (expectedY == "center") {
		expectedY = "50%";
	} else if (expectedY == "bottom") {
		expectedY = "100%";
	}
	expectedX = convertToPx(expectedX, divWidth);
	expectedY = convertToPx(expectedY, divHeight);
	if (expectedZ !== undefined) {
		expectedZ = convertToPx(expectedZ);
	} else {
		expectedZ = 0;
	}

	if (testTransformOrigin.counter === undefined) {
		testTransformOrigin.counter = 0;
	}
	// The transform doesn't matter here, so set it to one of several
	// possibilities arbitrarily (this actually catches a Gecko bug!)
	var transformValue = {
		0: "none",
		1: "matrix(7, 0, -1, 13, 0, 0)",
		2: "translate(4em, -15px)",
		3: "scale(1.2, 1)",
		4: "rotate(43deg)",
	}[testTransformOrigin.counter % 5];
	testTransformOrigin.counter++;
	div.removeAttribute("style");

	setStyles({transform: transformValue, transformOrigin: value});
	test(function() {
		var actual = getComputedStyle(div)[prefixProp("transformOrigin")];
		var re = expectedZ == 0
			? /^([^ ]+)px ([^ ]+)px$/
			: /^([^ ]+)px ([^ ]+)px ([^ ]+)px$/;
		assert_regexp_match(actual, re, "Computed value has unexpected form");
		var match = re.exec(actual);

		var msg = ' (actual: "' + actual + '", expected: "'
			+ expectedX + "px " + expectedY
			+ (expectedZ == 0 ? "" : "px " + expectedZ)
			+ 'px")';

		assert_approx_equals(Number(match[1]), expectedX, computedEpsilon,
			"Value of X part" + msg);

		assert_approx_equals(Number(match[2]), expectedY, computedEpsilon,
			"Value of Y part" + msg);

		if (expectedZ != 0) {
			assert_approx_equals(Number(match[3]), expectedZ, computedEpsilon,
				"Value of Z part" + msg);
		}
	}, "getComputedStyle(div).transformOrigin "
	+ getStyleDescription());

	// Test with a 45-degree rotation, since the effect of changing the origin
	// will be easy to understand.  In the 3D case, rotate around an
	// arbitrarily-chosen vector.
	testTransformedBoundary(
		// Transform
		expectedZ === 0
			? "rotate(45deg)"
			: "rotate3d(1,-1,1,45deg)",
		// Matrix entries
		expectedZ === 0
			? getRotationMatrix(0, 0, 1, "45deg")
			: getRotationMatrix(1, -1, 1, "45deg"),
		// Origin
		value, expectedX, expectedY, expectedZ
	);
}
//@}

/**
 * Tests that style="transform-origin: value" results in
 * getComputedStyle().transformOrigin being
 *   expectedX + "px " + expectedY + "px " + expectedZ + "px",
 * or if expectedZ is 0, just
 *   expectedX + "px " + expectedY + "px".
 */
function testTransformOriginParsing(expectedX, expectedY, expectedZ) {
//@{
}
//@}

/**
 * Tests that if perspective is set to value and perspective-origin is set to
 * originValue on div.parentNode, everything behaves as expected.  Tests
 * parsing of the perspective property, and boundaries of the test div.
 * expectedX and expectedY are strings to be passed to convertToPx(), so they
 * can be <length> or <percentage>.
 */
function testPerspective(value, originValue, expectedX, expectedY) {
//@{
	setStyles({}, {perspective: value});
	testPerspectiveParsing(value);

	// The expectedX/Y values used in the matrix need to be resolved relative
	// to the border box of the test div's parent, then offset by the
	// difference between their widths/heights.
	expectedX = convertToPx(expectedX, divParentWidth)
		- (divParentWidth - divWidth)/2;
	expectedY = convertToPx(expectedY, divParentHeight)
		- (divParentHeight - divHeight)/2;

	setStyles({}, {perspective: value, perspectiveOrigin: originValue});
	testPerspectiveBoundary(value, expectedX, expectedY);
	setStyles({}, {});
}
//@}

/**
 * Tests that getComputedStyle(div.parentNode).perspective is as one would
 * expect if it had the given perspective.
 */
function testPerspectiveParsing(value) {
//@{
	test(function() {
		var actual = getComputedStyle(div.parentNode)[prefixProp("perspective")];
		if (convertToPx(value) === null
		|| convertToPx(value) <= 0) {
			assert_equals(actual, "none");
			return;
		}
		assert_regexp_match(actual, /^[0-9]+(\.[0-9]+)?px$/, "Computed value has unexpected form");
		assert_approx_equals(parseFloat(actual), convertToPx(value), computedEpsilon);
	}, "getComputedStyle(div.parentNode).perspective " + getStyleDescription());
}
//@}

/**
 * Tests that the boundaries of the test div are as one would expect if its
 * parent had the given perspective and perspective-origin.  perspective will
 * be parsed to obtain the correct value, while perspectiveOriginX and
 * perspectiveOriginY must be numbers representing the pixel offset from the
 * *test div's* box (not its parent's).  perspectiveOriginX defaults to
 * divWidth/2, and perspectiveOriginY defaults to divHeight/2.
 */
function testPerspectiveBoundary(perspective, perspectiveOriginX, perspectiveOriginY) {
//@{
	if (convertToPx(perspective) === null || convertToPx(perspective) <= 0) {
		perspective = "none";
	}
	if (perspectiveOriginX === undefined) {
		perspectiveOriginX = divWidth/2;
	}
	if (perspectiveOriginY === undefined) {
		perspectiveOriginY = divHeight/2;
	}

	var mx = mxmul44(
		// Re-apply transform-origin of 50% 50%
		[1,0,0,0, 0,1,0,0, 0,0,1,0,
		-convertToPx("50%", divWidth), -convertToPx("50%", divHeight), 0, 1],
		// Un-apply perspective-origin
		[1,0,0,0, 0,1,0,0, 0,0,1,0, perspectiveOriginX, perspectiveOriginY, 0, 1],
		// Perspective
		[1,0,0,0, 0,1,0,0, 0,0,1,
		perspective == "none" ? 0 : -1/convertToPx(perspective),
		0,0,0,1],
		// Apply perspective-origin
		[1,0,0,0, 0,1,0,0, 0,0,1,0, -perspectiveOriginX, -perspectiveOriginY, 0, 1],
		// Un-apply transform-origin of 50% 50%
		[1,0,0,0, 0,1,0,0, 0,0,1,0,
		convertToPx("50%", divWidth), convertToPx("50%", divHeight), 0, 1],
		// Apply rotation of 45deg around X-axis
		getRotationMatrix(1, 0, 0, "45deg")
	);
	testTransformedBoundary("rotateX(45deg)", mx);
}
//@}

/**
 * Sets div.style.perspectiveOrigin to value, then tests that its computed
 * style is as expected.
 */
function testPerspectiveOrigin(value, expectedX, expectedY) {
//@{
	if (expectedX == "left") {
		expectedX = "0%";
	} else if (expectedX == "center") {
		expectedX = "50%";
	} else if (expectedX == "right") {
		expectedX = "100%";
	}
	if (expectedY == "top") {
		expectedY = "0%";
	} else if (expectedY == "center") {
		expectedY = "50%";
	} else if (expectedY == "bottom") {
		expectedY = "100%";
	}

	expectedX = convertToPx(expectedX, divWidth);
	expectedY = convertToPx(expectedY, divHeight);

	setStyles({perspectiveOrigin: value});
	test(function() {
		var actual = getComputedStyle(div)[prefixProp("perspectiveOrigin")];
		var re = /^([^ ]+)px ([^ ]+)px$/;
		assert_regexp_match(actual, re, "Computed value has unexpected form");
		var match = re.exec(actual);

		var msg = ' (actual: "' + actual + '", expected: "'
			+ expectedX + "px " + expectedY + 'px")';

		assert_approx_equals(Number(match[1]), expectedX, computedEpsilon,
			"Value of X part" + msg);

		assert_approx_equals(Number(match[2]), expectedY, computedEpsilon,
			"Value of Y part" + msg);
	}, "getComputedStyle(div).perspectiveOrigin " + getStyleDescription());

	setStyles({});
}
//@}

// vim: foldmarker=@{,@} foldmethod=marker
