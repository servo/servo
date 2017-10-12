"use strict";
function compareWidthHeight(id1, id2) {
  var element1 = document.getElementById(id1);
  var style1 = getComputedStyle(element1);
  var element2 = document.getElementById(id2);
  var style2 = getComputedStyle(element2);
  return (style1.width == style2.width) &&
      (style1.height == style2.height)
}
