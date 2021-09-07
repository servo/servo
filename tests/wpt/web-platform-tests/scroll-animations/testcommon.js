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
    currentScrollOffset, startScrollOffset, endScrollOffset) {
  return ((currentScrollOffset - startScrollOffset) /
          (endScrollOffset - startScrollOffset)) *
         100;
}

function createScroller(test) {
  var scroller = createDiv(test);
  scroller.innerHTML = "<div class='contents'></div>";
  scroller.classList.add('scroller');
  // Trigger layout run.
  scroller.scrollTop;
  return scroller;
}

function createScrollerWithStartAndEnd(test, orientationClass = 'vertical') {
  var scroller = createDiv(test);
  scroller.innerHTML =
    `<div class='contents'>
        <div id='start'></div>
        <div id='end'></div>
      </div>`;
  scroller.classList.add('scroller');
  scroller.classList.add(orientationClass);

  return scroller;
}

function createScrollTimeline(test, options) {
  options = options || {
    scrollSource: createScroller(test)
  }
  return new ScrollTimeline(options);
}

function createScrollTimelineWithOffsets(test, startOffset, endOffset) {
  return createScrollTimeline(test, {
    scrollSource: createScroller(test),
    orientation: "vertical",
    scrollOffsets: [startOffset, endOffset]
  });
}

function createScrollLinkedAnimation(test, timeline) {
  return createScrollLinkedAnimationWithTiming(test, /* duration in ms*/ 1000, timeline);
}

function createScrollLinkedAnimationWithTiming(test, timing, timeline) {
  if (timeline === undefined)
    timeline = createScrollTimeline(test);
  const KEYFRAMES = { opacity: [0, 1] };
  return new Animation(
    new KeyframeEffect(createDiv(test), KEYFRAMES, timing), timeline);
}

function assert_approx_equals_or_null(actual, expected, tolerance, name){
  if (actual === null || expected === null){
    assert_equals(actual, expected, name);
  }
  else {
    assert_approx_equals(actual, expected, tolerance, name);
  }
}

function assert_percents_equal(actual, expected, description){
  assert_equals(actual.unit, "percent", `'actual' unit type must be ` +
      `'percent' for "${description}"`);
  assert_true(actual instanceof CSSUnitValue, `'actual' must be of type ` +
      `CSSNumberish for "${description}"`);
  if (expected instanceof CSSUnitValue){
    // Verify that when the expected in a CSSUnitValue, it is the correct unit
    // type
    assert_equals(expected.unit, "percent", `'expected' unit type must be ` +
        `'percent' for "${description}"`);
    assert_approx_equals(actual.value, expected.value, 0.01, `values do not ` +
        `match for "${description}"`);
  } else if (typeof expected, "number"){
    assert_approx_equals(actual.value, expected, 0.01, `values do not match ` +
        `for "${description}"`);
  }
}

// These functions are used for the tests that have not yet been updated to be
// compatible with progress based scroll animations. Once scroll timeline
// "timeRange" is removed, these functions should also be removed.
// Needed work tracked by crbug.com/1216655
function createScrollTimelineWithTimeRange(test, options) {
  options = options || {
    scrollSource: createScroller(test),
    timeRange: 1000
  }
  return new ScrollTimeline(options);
}

function createScrollTimelineWithOffsetsWithTimeRange(test, startOffset, endOffset) {
  return createScrollTimelineWithTimeRange(test, {
    scrollSource: createScroller(test),
    orientation: "vertical",
    scrollOffsets: [startOffset, endOffset],
    timeRange: 1000
  });
}

function createScrollLinkedAnimationWithTimeRange(test, timeline) {
  if (timeline === undefined)
    timeline = createScrollTimelineWithTimeRange(test);
  const DURATION = 1000; // ms
  const KEYFRAMES = { opacity: [0, 1] };
  return new Animation(
    new KeyframeEffect(createDiv(test), KEYFRAMES, DURATION), timeline);
}