'use strict';

function assert_px_equals(observed, expected, description) {
  assert_equals(observed.unit, 'px',
                `Unexpected unit type for '${description}'`);
  assert_approx_equals(observed.value, expected, 0.0001,
                       `Unexpected value for ${description}`);
}

function CreateViewTimelineOpacityAnimation(test, target, options) {
  const viewTimelineOptions = {
    subject: target,
    axis: 'block'
  };
  if (options) {
    for (let key in options) {
      viewTimelineOptions[key] = options[key];
    }
  }

  const anim =
      target.animate(
          { opacity: [0.3, 0.7] },
          { timeline: new ViewTimeline(viewTimelineOptions) });
  test.add_cleanup(() => {
    anim.cancel();
  });
  return anim;
}

// Verify that range specified in the options aligns with the active range of
// the animation.
//
// Sample call:
// await runTimelineRangeTest(t, {
//   timeline: { inset: [ CSS.percent(0), CSS.percent(20)] },
//   timing: { fill: 'both' }
//   rangeStart: 600,
//   rangeEnd: 900
// });
async function runTimelineRangeTest(t, options, message) {
  container.scrollLeft = 0;
  await waitForNextFrame();

  const anim =
      options.anim ||
      CreateViewTimelineOpacityAnimation(t, target, options.timeline);
  if (options.timing)
    anim.effect.updateTiming(options.timing);

  const timeline = anim.timeline;
  await anim.ready;

  // Advance to the start offset, which triggers entry to the active phase.
  container.scrollLeft = options.rangeStart;
  await waitForNextFrame();
  assert_equals(getComputedStyle(target).opacity, '0.3',
                `Effect at the start of the active phase: ${message}`);

  // Advance to the midpoint of the animation.
  container.scrollLeft = (options.rangeStart + options.rangeEnd) / 2;
  await waitForNextFrame();
  assert_equals(getComputedStyle(target).opacity,'0.5',
                `Effect at the midpoint of the active range: ${message}`);

  // Advance to the end of the animation.
  container.scrollLeft = options.rangeEnd;
  await waitForNextFrame();
  assert_equals(getComputedStyle(target).opacity, '0.7',
                `Effect is in the active phase at effect end time: ${message}`);

  // Return the animation so that we can continue testing with the same object.
  return anim;
}

// Sets the start and end delays for a view timeline and ensures that the
// range aligns with expected values.
//
// Sample call:
// await runTimelineDelayTest(t, {
//   delay: { phase: 'cover', percent: CSS.percent(0) } ,
//   endDelay: { phase: 'cover', percent: CSS.percent(100) },
//   rangeStart: 600,
//   rangeEnd: 900
// });
async function runTimelineDelayTest(t, options) {
  const delayToString = delay => {
    const parts = [];
    if (delay.phase)
      parts.push(delay.phase);
    if (delay.percent)
      parts.push(`${delay.percent.value}%`);
    return parts.join(' ');
  };
  const range =
     `${delayToString(options.delay)} to ` +
     `${delayToString(options.endDelay)}`;

  options.timeline = {
    axis: 'inline'
  };
  options.timing = {
    delay: options.delay,
    endDelay: options.endDelay,
    // Set fill to accommodate floating point precision errors at the
    // endpoints.
    fill: 'both'
  };

  return runTimelineRangeTest(t, options, range);
}

// Sets the Inset for a view timeline and ensures that the range aligns with
// expected values.
//
// Sample call:
// await runTimelineDelayTest(t, {
//   inset: [ CSS.px(20), CSS.px(40) ]
//   rangeStart: 600,
//   rangeEnd: 900
// });
async function runTimelineInsetTest(t, options) {
  options.timeline = {
    axis: 'inline',
    inset: options.inset
  };
  options.timing = {
    // Set fill to accommodate floating point precision errors at the
    // endpoints.
    fill: 'both'
  }
  const length = options.inset.length;
  const range =
      (options.inset instanceof Array) ? options.inset.join(' ')
                                       : options.inset;
  return runTimelineRangeTest(t, options, range);
}
