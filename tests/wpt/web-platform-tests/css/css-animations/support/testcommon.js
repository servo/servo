/* Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/ */

/**
 * Use this variable if you specify duration or some other properties
 * for script animation.
 * E.g., div.animate({ opacity: [0, 1] }, 100 * MS_PER_SEC);
 *
 * NOTE: Creating animations with short duration may cause intermittent
 * failures in asynchronous test. For example, the short duration animation
 * might be finished when animation.ready has been fulfilled because of slow
 * platforms or busyness of the main thread.
 * Setting short duration to cancel its animation does not matter but
 * if you don't want to cancel the animation, consider using longer duration.
 */
const MS_PER_SEC = 1000;

/* The recommended minimum precision to use for time values[1].
 *
 * [1] https://drafts.csswg.org/web-animations/#precision-of-time-values
 */
var TIME_PRECISION = 0.0005; // ms

/*
 * Allow implementations to substitute an alternative method for comparing
 * times based on their precision requirements.
 */
function assert_times_equal(actual, expected, description) {
  assert_approx_equals(actual, expected, TIME_PRECISION * 2, description);
}

/*
 * Compare a time value based on its precision requirements with a fixed value.
 */
function assert_time_equals_literal(actual, expected, description) {
  assert_approx_equals(actual, expected, TIME_PRECISION, description);
}

/**
 * Appends a div to the document body.
 *
 * @param t  The testharness.js Test object. If provided, this will be used
 *           to register a cleanup callback to remove the div when the test
 *           finishes.
 *
 * @param attrs  A dictionary object with attribute names and values to set on
 *               the div.
 */
function addDiv(t, attrs) {
  var div = document.createElement('div');
  if (attrs) {
    for (var attrName in attrs) {
      div.setAttribute(attrName, attrs[attrName]);
    }
  }
  document.body.appendChild(div);
  if (t && typeof t.add_cleanup === 'function') {
    t.add_cleanup(function() {
      if (div.parentNode) {
        div.remove();
      }
    });
  }
  return div;
}

/**
 * Appends a style div to the document head.
 *
 * @param t  The testharness.js Test object. If provided, this will be used
 *           to register a cleanup callback to remove the style element
 *           when the test finishes.
 *
 * @param rules  A dictionary object with selector names and rules to set on
 *               the style sheet.
 */
function addStyle(t, rules) {
  var extraStyle = document.createElement('style');
  document.head.appendChild(extraStyle);
  if (rules) {
    var sheet = extraStyle.sheet;
    for (var selector in rules) {
      sheet.insertRule(selector + '{' + rules[selector] + '}',
                       sheet.cssRules.length);
    }
  }

  if (t && typeof t.add_cleanup === 'function') {
    t.add_cleanup(function() {
      extraStyle.remove();
    });
  }
}

/**
 * Promise wrapper for requestAnimationFrame.
 */
function waitForFrame() {
  return new Promise(function(resolve, reject) {
    window.requestAnimationFrame(resolve);
  });
}

/**
 * Waits for a requestAnimationFrame callback in the next refresh driver tick.
 */
function waitForNextFrame() {
  const timeAtStart = document.timeline.currentTime;
  return new Promise(resolve => {
    window.requestAnimationFrame(() => {
      if (timeAtStart === document.timeline.currentTime) {
        window.requestAnimationFrame(resolve);
      } else {
        resolve();
      }
    });
  });
}

/**
 * Returns a Promise that is resolved after the given number of consecutive
 * animation frames have occured (using requestAnimationFrame callbacks).
 *
 * @param frameCount  The number of animation frames.
 * @param onFrame  An optional function to be processed in each animation frame.
 */
function waitForAnimationFrames(frameCount, onFrame) {
  const timeAtStart = document.timeline.currentTime;
  return new Promise(function(resolve, reject) {
    function handleFrame() {
      if (onFrame && typeof onFrame === 'function') {
        onFrame();
      }
      if (timeAtStart != document.timeline.currentTime &&
          --frameCount <= 0) {
        resolve();
      } else {
        window.requestAnimationFrame(handleFrame); // wait another frame
      }
    }
    window.requestAnimationFrame(handleFrame);
  });
}

/**
 * Wrapper that takes a sequence of N animations and returns:
 *
 *   Promise.all([animations[0].ready, animations[1].ready, ... animations[N-1].ready]);
 */
function waitForAllAnimations(animations) {
  return Promise.all(animations.map(animation => animation.ready));
}

/**
 * Flush the computed style for the given element. This is useful, for example,
 * when we are testing a transition and need the initial value of a property
 * to be computed so that when we synchronouslyet set it to a different value
 * we actually get a transition instead of that being the initial value.
 */
function flushComputedStyle(elem) {
  var cs = getComputedStyle(elem);
  cs.marginLeft;
}
// Waits for a given animation being ready to restyle.
async function waitForAnimationReadyToRestyle(aAnimation) {
  await aAnimation.ready;
  // If |aAnimation| begins at the current timeline time, we will not process
  // restyling in the initial frame because of aligning with the refresh driver,
  // the animation frame in which the ready promise is resolved happens to
  // coincide perfectly with the start time of the animation.  In this case no
  // restyling is needed in the frame so we have to wait one more frame.
  if (animationStartsRightNow(aAnimation)) {
    await waitForNextFrame();
  }
}
