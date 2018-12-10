'use strict';

// Builds a generic structure that looks like:
//
// <div class="scroller">  // 100x100 viewport
//   <div class="contents"></div>  // 500x500
// </div>
//
// The |scrollerOverrides| and |contentOverrides| parameters are maps which
// are applied to the scroller and contents style after basic setup.
//
// Appends the outer 'scroller' element to the document body, and returns it.
function setupScrollTimelineTest(
    scrollerOverrides = new Map(), contentOverrides = new Map()) {
  let scroller = document.createElement('div');
  scroller.style.width = '100px';
  scroller.style.height = '100px';
  scroller.style.overflow = 'scroll';
  for (const [key, value] of scrollerOverrides) {
    scroller.style[key] = value;
  }

  let contents = document.createElement('div');
  contents.style.width = '500px';
  contents.style.height = '500px';
  for (const [key, value] of contentOverrides) {
    contents.style[key] = value;
  }

  scroller.appendChild(contents);
  document.body.appendChild(scroller);
  return scroller;
}

// Helper method to calculate the current time, implementing only step 5 of
// https://wicg.github.io/scroll-animations/#current-time-algorithm
function calculateCurrentTime(
    currentScrollOffset, startScrollOffset, endScrollOffset,
    effectiveTimeRange) {
  return ((currentScrollOffset - startScrollOffset) /
          (endScrollOffset - startScrollOffset)) *
      effectiveTimeRange;
}

