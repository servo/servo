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
    scrollSource: createScroller(test),
    timeRange: 1000
  }
  return new ScrollTimeline(options);
}

function createScrollTimelineWithOffsets(test, startOffset, endOffset) {
  return createScrollTimeline(test, {
    scrollSource: createScroller(test),
    orientation: "vertical",
    startScrollOffset: startOffset,
    endScrollOffset: endOffset,
    timeRange: 1000
  });
}

function createScrollLinkedAnimation(test, timeline) {
  if (timeline === undefined)
    timeline = createScrollTimeline(test);
  const DURATION = 1000; // ms
  const KEYFRAMES = { opacity: [0, 1] };
  return new Animation(
    new KeyframeEffect(createDiv(test), KEYFRAMES, DURATION), timeline);
}

function assert_approx_equals_or_null(actual, expected, tolerance, name){
  if (actual === null || expected === null){
    assert_equals(actual, expected, name);
  }
  else {
    assert_approx_equals(actual, expected, tolerance, name);
  }
}

// actual should be a CSSUnitValue and expected should be a double value 0-100
function assert_percent_css_unit_value_approx_equals(actual, expected, tolerance, name){
  assert_true(actual instanceof CSSUnitValue, "'actual' must be of type CSSUnitValue");
  assert_equals(typeof expected, "number", "'expected' should be a number (0-100)");
  assert_equals(actual.unit, "percent", "'actual' unit type must be 'percent'");
  assert_approx_equals(actual.value, expected, tolerance, name);
}