var selfPositionValues = [ "start", "end", "self-start", "self-end", "left", "right", "center", "flex-start", "flex-end"];
var contentPositionValues = [ "start", "end", "left", "right", "center", "flex-start", "flex-end"];
var distributionValues = [ "stretch", "space-around", "space-between", "space-evenly"];
var baselineValues = [ "baseline", "first baseline", "last baseline"];

function checkPlaceShorhand(shorthand, alignValue, justifyValue)
{
    var div = document.createElement("div");
    var specifiedValue = (alignValue + " " + justifyValue).trim();
    div.style[shorthand] = specifiedValue;
    document.body.appendChild(div);

    if (alignValue === justifyValue)
        specifiedValue = alignValue;

    var resolvedValue = getComputedStyle(div).getPropertyValue(shorthand);
    if (alignValue === "first baseline")
        alignValue = "baseline";
    if (justifyValue === "first baseline")
        justifyValue = "baseline";
    if (justifyValue === "")
        justifyValue = alignValue;
    var expectedResolvedValue = (alignValue + " " + justifyValue).trim()

    assert_equals(div.style[shorthand], specifiedValue, shorthand + " specified value");
    // FIXME: We need https://github.com/w3c/csswg-drafts/issues/1041 to clarify which
    // value is expected for the shorthand's 'resolved value".
    assert_in_array(resolvedValue, ["", expectedResolvedValue], shorthand + " resolved value");
}

function checkPlaceShorhandLonghands(shorthand, alignLonghand, justifyLonghand, alignValue, justifyValue = "")
{
    var div = document.createElement("div");
    div.setAttribute("style", shorthand + ": " + alignValue + " " + justifyValue);
    document.body.appendChild(div);
    if (justifyValue === "")
        justifyValue = alignValue;
    assert_equals(div.style[alignLonghand],
                  alignValue, alignLonghand + " expanded value");
    assert_equals(div.style[justifyLonghand],
                  justifyValue, justifyLonghand + " expanded value");
}

function checkPlaceShorthandInvalidValues(shorthand, alignLonghand, justifyLonghand, value)
{
    var div = document.createElement("div");
    var css = alignLonghand + ": start; " + justifyLonghand + ": end;" + shorthand + ": " + value;
    div.setAttribute("style", css);
    document.body.appendChild(div);
    assert_equals(div.style[alignLonghand],
                  "start", alignLonghand + " expanded value");
    assert_equals(div.style[justifyLonghand],
                  "end", justifyLonghand + " expanded value");
}
