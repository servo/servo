/**
 * The function positions a new div to exactly the bounding client rect without
 * using sticky position. If it's directly under the sticky element it could be
 * obscured and not show up when compared to the ref.  */
function createIndicatorForStickyElements(sticky_divs) {
  sticky_divs.forEach((sticky_div) => {
    // The relative position indicator will be able to share the same containing
    // block to match the position with the same offset from in flow position
    // (offsetTop/offsetLeft)
    if (getComputedStyle(sticky_div).position != "sticky")
      throw "Provided sticky element does not have position: sticky";
    var position_div = document.createElement("div");
    position_div.style.left = sticky_div.offsetLeft + "px";
    position_div.style.top  = sticky_div.offsetTop + "px";
    // The absolute position is to ensure that the position_div adds zero size
    // to in flow layout
    position_div.style.position = "absolute"
    var indicator_div = document.createElement("div");
    indicator_div.style.width = sticky_div.offsetWidth + "px";
    indicator_div.style.height = sticky_div.offsetHeight + "px";
    indicator_div.style.backgroundColor = "blue";
    indicator_div.style.position = "relative";
    position_div.appendChild(indicator_div);
    sticky_div.parentNode.insertBefore(position_div, sticky_div);
  });
}
