/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*** State *******************************************************************/

window.COLORS = [
  "#0088cc",
  "#5b5fff",
  "#b82ee5",
  "#ed2655",
  "#f13c00",
  "#d97e00",
  "#2cbb0f",
  "#0072ab",
];

window.MIN_TRACE_TIME = 100000; // .1 ms

// A class containing the cleaned up trace state.
window.State = (function () {
  return class {
    constructor() {
      // The traces themselves.
      this.traces = null;

      // Maximimum and minimum times seen in traces. These get normalized to be
      // relative to 0, so after initialization minTime is always 0.
      this.minTime = Infinity;
      this.maxTime = 0;

      // The current start and end of the viewport selection.
      this.startSelection = 0;
      this.endSelection = 0;

      // The current width of the window.
      this.windowWidth = window.innerWidth;

      // Whether the user is actively grabbing the left or right grabby, or the
      // viewport slider.
      this.grabbingLeft = false;
      this.grabbingRight = false;
      this.grabbingSlider = false;

      // Maps category labels to a persistent color so that they are always
      // rendered the same color.
      this.colorIndex = 0;
      this.categoryToColor = Object.create(null);

      this.initialize();
    }

    // Clean up and massage the trace data.
    initialize() {
      this.traces = TRACES.filter(t => t.endTime - t.startTime >= MIN_TRACE_TIME);
      window.TRACES = null;

      this.traces.sort((t1, t2) => {
        let cmp = t1.startTime - t2.startTime;
        if (cmp !== 0) {
          return cmp;
        }

        return t1.endTime - t2.endTime;
      });

      this.findMinTime();
      this.normalizeTimes();
      this.removeIdleTime();
      this.findMaxTime();

      this.startSelection = 3 * this.maxTime / 8;
      this.endSelection = 5 * this.maxTime / 8;
    }

    // Find the minimum timestamp.
    findMinTime() {
      this.minTime = this.traces.reduce((min, t) => Math.min(min, t.startTime),
                                        Infinity);
    }

    // Find the maximum timestamp.
    findMaxTime() {
      this.maxTime = this.traces.reduce((max, t) => Math.max(max, t.endTime),
                                        0);
    }

    // Normalize all times to be relative to the minTime and then reset the
    // minTime to 0.
    normalizeTimes() {
      for (let i = 0; i < this.traces.length; i++) {
        let trace = this.traces[i];
        trace.startTime -= this.minTime;
        trace.endTime -= this.minTime;
      }
      this.minTime = 0;
    }

    // Remove idle time between traces. It isn't useful to see and makes
    // visualizing the data more difficult.
    removeIdleTime() {
      let totalIdleTime = 0;
      let lastEndTime = null;

      for (let i = 0; i < this.traces.length; i++) {
        let trace = this.traces[i];

        if (lastEndTime !== null && trace.startTime > lastEndTime) {
          totalIdleTime += trace.startTime - lastEndTime;
        }

        lastEndTime = trace.endTime;

        trace.startTime -= totalIdleTime;
        trace.endTime -= totalIdleTime;
      }
    }

    // Get the color for the given category, or assign one if no such color
    // exists yet.
    getColorForCategory(category) {
      let result = this.categoryToColor[category];
      if (!result) {
        result = COLORS[this.colorIndex++ % COLORS.length];
        this.categoryToColor[category] = result;
      }
      return result;
    }
  };
}());

window.state = new State();

/*** Utilities ****************************************************************/

// Get the closest power of ten to the given number.
window.closestPowerOfTen = n => {
  let powerOfTen = 1;
  let diff = Math.abs(n - powerOfTen);

  while (true) {
    let nextPowerOfTen = powerOfTen * 10;
    let nextDiff = Math.abs(n - nextPowerOfTen);

    if (nextDiff > diff) {
      return powerOfTen;
    }

    diff = nextDiff;
    powerOfTen = nextPowerOfTen;
  }
};

// Select the tick increment for the given range size and maximum number of
// ticks to show for that range.
window.selectIncrement = (range, maxTicks) => {
  let increment = closestPowerOfTen(range / 10);
  while (range / increment > maxTicks) {
    increment *= 2;
  }
  return increment;
};

// Get the category name for the given trace.
window.traceCategory = trace => {
  return Object.keys(trace.category)[0];
};

/*** Initial Persistent Element Creation **************************************/

document.body.innerHTML = "";

window.sliderContainer = document.createElement("div");
sliderContainer.id = "slider";
document.body.appendChild(sliderContainer);

window.leftGrabby = document.createElement("span");
leftGrabby.className = "grabby";
sliderContainer.appendChild(leftGrabby);

window.sliderViewport = document.createElement("span");
sliderViewport.id = "slider-viewport";
sliderContainer.appendChild(sliderViewport);

window.rightGrabby = document.createElement("span");
rightGrabby.className = "grabby";
sliderContainer.appendChild(rightGrabby);

window.tracesContainer = document.createElement("div");
tracesContainer.id = "traces";
document.body.appendChild(tracesContainer);

/*** Listeners ***************************************************************/

// Run the given function and render afterwards.
window.withRender = fn => (...args) => {
  fn(...args);
  render();
};

window.addEventListener("resize", withRender(() => {
  state.windowWidth = window.innerWidth;
}));

window.addEventListener("mouseup", () => {
  state.grabbingSlider = state.grabbingLeft = state.grabbingRight = false;
});

leftGrabby.addEventListener("mousedown", () => {
  state.grabbingLeft = true;
});

rightGrabby.addEventListener("mousedown", () => {
  state.grabbingRight = true;
});

sliderViewport.addEventListener("mousedown", () => {
  state.grabbingSlider = true;
});

window.addEventListener("mousemove", event => {
  let ratio = event.clientX / state.windowWidth;
  let relativeTime = ratio * state.maxTime;
  let absTime = state.minTime + relativeTime;
  absTime = Math.min(state.maxTime, absTime);
  absTime = Math.max(state.minTime, absTime);

  if (state.grabbingSlider) {
    let delta = event.movementX / state.windowWidth * state.maxTime;
    if (delta < 0) {
      delta = Math.max(-state.startSelection, delta);
    } else {
      delta = Math.min(state.maxTime - state.endSelection, delta);
    }

    state.startSelection += delta;
    state.endSelection += delta;
    render();
  } else if (state.grabbingLeft) {
    state.startSelection = Math.min(absTime, state.endSelection);
    render();
  } else if (state.grabbingRight) {
    state.endSelection = Math.max(absTime, state.startSelection);
    render();
  }
});

sliderContainer.addEventListener("wheel", withRender(event => {
  let increment = state.maxTime / 1000;

  state.startSelection -= event.deltaY * increment
  state.startSelection = Math.max(0, state.startSelection);
  state.startSelection = Math.min(state.startSelection, state.endSelection);

  state.endSelection += event.deltaY * increment;
  state.endSelection = Math.min(state.maxTime, state.endSelection);
  state.endSelection = Math.max(state.startSelection, state.endSelection);
}));

/*** Rendering ***************************************************************/

// Create a function that calls the given function `fn` only once per animation
// frame.
window.oncePerAnimationFrame = fn => {
  let animationId = null;
  return () => {
    if (animationId !== null) {
      return;
    }

    animationId = requestAnimationFrame(() => {
      fn();
      animationId = null;
    });
  };
};

// Only call the given function once per window width resize.
window.oncePerWindowWidth = fn => {
  let lastWidth = null;
  return () => {
    if (state.windowWidth !== lastWidth) {
      fn();
      lastWidth = state.windowWidth;
    }
  };
};

// Top level entry point for rendering. Renders the current `window.state`.
window.render = oncePerAnimationFrame(() => {
  renderSlider();
  renderTraces();
});

// Render the slider at the top of the screen.
window.renderSlider = () => {
  let selectionDelta = state.endSelection - state.startSelection;

  leftGrabby.style.marginLeft = (state.startSelection / state.maxTime) * state.windowWidth + "px";

  // -6px because of the 3px width of each grabby.
  sliderViewport.style.width = (selectionDelta / state.maxTime) * state.windowWidth - 6 + "px";

  rightGrabby.style.rightMargin = (state.maxTime - state.endSelection) / state.maxTime
                                * state.windowWidth + "px";

  renderSliderTicks();
};

// Render the ticks along the slider overview.
window.renderSliderTicks = oncePerWindowWidth(() => {
  let oldTicks = Array.from(document.querySelectorAll(".slider-tick"));
  for (let tick of oldTicks) {
    tick.remove();
  }

  let increment = selectIncrement(state.maxTime, 20);
  let px = increment / state.maxTime * state.windowWidth;
  let ms = 0;
  for (let i = 0; i < state.windowWidth; i += px) {
    let tick = document.createElement("div");
    tick.className = "slider-tick";
    tick.textContent = ms + " ms";
    tick.style.left = i + "px";
    document.body.appendChild(tick);
    ms += increment / 1000000;
  }
});

// Render the individual traces.
window.renderTraces = () => {
  renderTracesTicks();

  let tracesToRender = [];
  for (let i = 0; i < state.traces.length; i++) {
    let trace = state.traces[i];

    if (trace.endTime < state.startSelection || trace.startTime > state.endSelection) {
      continue;
    }

    tracesToRender.push(trace);
  }

  // Ensure that we have enouch traces elements. If we have more elements than
  // traces we are going to render, then remove some. If we have fewer elements
  // than traces we are going to render, then add some.
  let rows = Array.from(tracesContainer.querySelectorAll(".outer"));
  while (rows.length > tracesToRender.length) {
    rows.pop().remove();
  }
  while (rows.length < tracesToRender.length) {
    let elem = makeTraceTemplate();
    tracesContainer.appendChild(elem);
    rows.push(elem);
  }

  for (let i = 0; i < tracesToRender.length; i++) {
    renderTrace(tracesToRender[i], rows[i]);
  }
};

// Render the ticks behind the traces.
window.renderTracesTicks = () => {
  let oldTicks = Array.from(tracesContainer.querySelectorAll(".traces-tick"));
  for (let tick of oldTicks) {
    tick.remove();
  }

  let selectionDelta = state.endSelection - state.startSelection;
  let increment = selectIncrement(selectionDelta, 10);
  let px = increment / selectionDelta * state.windowWidth;
  let offset = state.startSelection % increment;
  let time = state.startSelection - offset + increment;

  while (time < state.endSelection) {
    let tick = document.createElement("div");
    tick.className = "traces-tick";
    tick.textContent = Math.round(time / 1000000) + " ms";
    tick.style.left = (time - state.startSelection) / selectionDelta * state.windowWidth + "px";
    tracesContainer.appendChild(tick);

    time += increment;
  }
};

// Create the DOM structure for an individual trace.
window.makeTraceTemplate = () => {
  let outer = document.createElement("div");
  outer.className = "outer";

  let inner = document.createElement("div");
  inner.className = "inner";

  let tooltip = document.createElement("div");
  tooltip.className = "tooltip";

  let header = document.createElement("h3");
  header.className = "header";
  tooltip.appendChild(header);

  let duration = document.createElement("h4");
  duration.className = "duration";
  tooltip.appendChild(duration);

  let pairs = document.createElement("dl");

  let timeStartLabel = document.createElement("dt");
  timeStartLabel.textContent = "Start:"
  pairs.appendChild(timeStartLabel);

  let timeStartValue = document.createElement("dd");
  timeStartValue.className = "start";
  pairs.appendChild(timeStartValue);

  let timeEndLabel = document.createElement("dt");
  timeEndLabel.textContent = "End:"
  pairs.appendChild(timeEndLabel);

  let timeEndValue = document.createElement("dd");
  timeEndValue.className = "end";
  pairs.appendChild(timeEndValue);

  let urlLabel = document.createElement("dt");
  urlLabel.textContent = "URL:";
  pairs.appendChild(urlLabel);

  let urlValue = document.createElement("dd");
  urlValue.className = "url";
  pairs.appendChild(urlValue);

  let iframeLabel = document.createElement("dt");
  iframeLabel.textContent = "iframe?";
  pairs.appendChild(iframeLabel);

  let iframeValue = document.createElement("dd");
  iframeValue.className = "iframe";
  pairs.appendChild(iframeValue);

  let incrementalLabel = document.createElement("dt");
  incrementalLabel.textContent = "Incremental?";
  pairs.appendChild(incrementalLabel);

  let incrementalValue = document.createElement("dd");
  incrementalValue.className = "incremental";
  pairs.appendChild(incrementalValue);

  tooltip.appendChild(pairs);
  outer.appendChild(tooltip);
  outer.appendChild(inner);
  return outer;
};

// Render `trace` into the given `elem`. We reuse the trace elements and modify
// them with the new trace that will populate this particular `elem` rather than
// clearing the DOM out and rebuilding it from scratch. Its a bit of a
// performance win when there are a lot of traces being rendered. Funnily
// enough, iterating over the complete set of traces hasn't been a performance
// problem at all and the bottleneck seems to be purely rendering the subset of
// traces we wish to show.
window.renderTrace = (trace, elem) => {
  let inner = elem.querySelector(".inner");
  inner.style.width = (trace.endTime - trace.startTime) / (state.endSelection - state.startSelection)
                    * state.windowWidth + "px";
  inner.style.marginLeft = (trace.startTime - state.startSelection)
                         / (state.endSelection - state.startSelection)
                         * state.windowWidth + "px";

  let category = traceCategory(trace);
  inner.textContent = category;
  inner.style.backgroundColor = state.getColorForCategory(category);

  let header = elem.querySelector(".header");
  header.textContent = category;

  let duration = elem.querySelector(".duration");
  duration.textContent = (trace.endTime - trace.startTime) / 1000000 + " ms";

  let timeStartValue = elem.querySelector(".start");
  timeStartValue.textContent = trace.startTime / 1000000 + " ms";

  let timeEndValue = elem.querySelector(".end");
  timeEndValue.textContent = trace.endTime / 1000000 + " ms";

  if (trace.metadata) {
    let urlValue = elem.querySelector(".url");
    urlValue.textContent = trace.metadata.url;
    urlValue.removeAttribute("hidden");

    let iframeValue = elem.querySelector(".iframe");
    iframeValue.textContent = trace.metadata.iframe.RootWindow ? "No" : "Yes";
    iframeValue.removeAttribute("hidden");

    let incrementalValue = elem.querySelector(".incremental");
    incrementalValue.textContent = trace.metadata.incremental.Incremental ? "Yes" : "No";
    incrementalValue.removeAttribute("hidden");
  } else {
    elem.querySelector(".url").setAttribute("hidden", "");
    elem.querySelector(".iframe").setAttribute("hidden", "");
    elem.querySelector(".incremental").setAttribute("hidden", "");
  }
};

render();
