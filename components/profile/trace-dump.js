/*
!!! THIS FILE IS COPIED FROM https://github.com/fitzgen/servo-trace-dump !!!
Make sure to upstream changes, or they will get lost!
*/
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
"use strict";

(function (exports, window) {

  /*** State ******************************************************************/

  const COLORS = exports.COLORS = [
    "#0088cc",
    "#5b5fff",
    "#b82ee5",
    "#ed2655",
    "#f13c00",
    "#d97e00",
    "#2cbb0f",
    "#0072ab",
  ];

  // A class containing the cleaned up trace state.
  const State = exports.State = (function () {
    return class State {
      constructor(rawTraces, windowWidth) {
        // The traces themselves.
        this.traces = null;

        // Only display traces that take at least this long. Default is .1 ms.
        this.minimumTraceTime = 100000;

        // Maximimum and minimum times seen in traces. These get normalized to be
        // relative to 0, so after initialization minTime is always 0.
        this.minTime = Infinity;
        this.maxTime = 0;

        // The current start and end of the viewport selection.
        this.startSelection = 0;
        this.endSelection = 0;

        // The current width of the window.
        this.windowWidth = windowWidth;

        // Whether the user is actively grabbing the left or right grabby, or the
        // viewport slider.
        this.grabbingLeft = false;
        this.grabbingRight = false;
        this.grabbingSlider = false;

        // Maps category labels to a persistent color so that they are always
        // rendered the same color.
        this.colorIndex = 0;
        this.categoryToColor = Object.create(null);

        this.initialize(rawTraces);
      }

      // Clean up and massage the trace data.
      initialize(rawTraces) {
        this.traces = rawTraces.filter(t => t.endTime - t.startTime >= this.minimumTraceTime);

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

      // Translate pixels into nanoseconds.
      pxToNs(px) {
        return px / this.windowWidth * this.maxTime;
      }

      // Translate nanoseconds into pixels.
      nsToPx(ns) {
        return ns / this.maxTime * this.windowWidth
      }

      // Translate nanoseconds into pixels in the zoomed viewport region.
      nsToSelectionPx(ns) {
        return ns / (this.endSelection - this.startSelection) * this.windowWidth;
      }

      // Update the start selection to the given position's time.
      updateStartSelection(position) {
        this.startSelection = clamp(this.pxToNs(position),
                                    this.minTime,
                                    this.endSelection);
      }

      // Update the end selection to the given position's time.
      updateEndSelection(position) {
        this.endSelection = clamp(this.pxToNs(position),
                                  this.startSelection,
                                  this.maxTime);
      }

      // Move the start and end selection by the given delta movement.
      moveSelection(movement) {
        let delta = clamp(this.pxToNs(movement),
                          -this.startSelection,
                          this.maxTime - this.endSelection);

        this.startSelection += delta;
        this.endSelection += delta;
      }

      // Widen or narrow the selection based on the given zoom.
      zoomSelection(zoom) {
        const increment = this.maxTime / 1000;

        this.startSelection = clamp(this.startSelection - zoom * increment,
                                    this.minTime,
                                    this.endSelection);

        this.endSelection = clamp(this.endSelection + zoom * increment,
                                  this.startSelection,
                                  this.maxTime);
      }

      // Get the set of traces that overlap the current selection.
      getTracesInSelection() {
        const tracesInSelection = [];

        for (let i = 0; i < state.traces.length; i++) {
          let trace = state.traces[i];

          if (trace.endTime < state.startSelection) {
            continue;
          }

          if (trace.startTime > state.endSelection) {
            break;
          }

          tracesInSelection.push(trace);
        }

        return tracesInSelection;
      }
    };
  }());

  /*** Utilities **************************************************************/

  // Ensure that min <= value <= max
  const clamp = exports.clamp = (value, min, max) => {
    return Math.max(Math.min(value, max), min);
  };

  // Get the closest power of ten to the given number.
  const closestPowerOfTen = exports.closestPowerOfTen = n => {
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
  const selectIncrement = exports.selectIncrement = (range, maxTicks) => {
    let increment = closestPowerOfTen(range / 10);
    while (range / increment > maxTicks) {
      increment *= 2;
    }
    return increment;
  };

  /*** Window Specific Code ***************************************************/

  if (!window) {
    return;
  }

  // XXX: Everything below here relies on the presence of `window`! Try to
  // minimize this code and factor out the parts that don't explicitly need
  // `window` or `document` as much as possible. We can't easily test code that
  // relies upon `window`.

  const state = exports.state = new State(window.TRACES, window.innerWidth);

  /*** Initial Persistent Element Creation ************************************/

  window.document.body.innerHTML = "";

  const sliderContainer = window.document.createElement("div");
  sliderContainer.id = "slider";
  window.document.body.appendChild(sliderContainer);

  const leftGrabby = window.document.createElement("span");
  leftGrabby.className = "grabby";
  sliderContainer.appendChild(leftGrabby);

  const sliderViewport = window.document.createElement("span");
  sliderViewport.id = "slider-viewport";
  sliderContainer.appendChild(sliderViewport);

  const rightGrabby = window.document.createElement("span");
  rightGrabby.className = "grabby";
  sliderContainer.appendChild(rightGrabby);

  const tracesContainer = window.document.createElement("div");
  tracesContainer.id = "traces";
  window.document.body.appendChild(tracesContainer);

  /*** Listeners *************************************************************/

  // Run the given function and render afterwards.
  const withRender = fn => function () {
    fn.apply(null, arguments);
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
    if (state.grabbingSlider) {
      state.moveSelection(event.movementX);
      event.preventDefault();
      render();
    } else if (state.grabbingLeft) {
      state.updateStartSelection(event.clientX);
      event.preventDefault();
      render();
    } else if (state.grabbingRight) {
      state.updateEndSelection(event.clientX);
      event.preventDefault();
      render();
    }
  });

  sliderContainer.addEventListener("wheel", withRender(event => {
    state.zoomSelection(event.deltaY);
  }));

  /*** Rendering **************************************************************/

  // Create a function that calls the given function `fn` only once per animation
  // frame.
  const oncePerAnimationFrame = fn => {
    let animationId = null;
    return () => {
      if (animationId !== null) {
        return;
      }

      animationId = window.requestAnimationFrame(() => {
        fn();
        animationId = null;
      });
    };
  };

  // Only call the given function once per window width resize.
  const oncePerWindowWidth = fn => {
    let lastWidth = null;
    return () => {
      if (state.windowWidth !== lastWidth) {
        fn();
        lastWidth = state.windowWidth;
      }
    };
  };

  // Top level entry point for rendering. Renders the current `window.state`.
  const render = oncePerAnimationFrame(() => {
    renderSlider();
    renderTraces();
  });

  // Render the slider at the top of the screen.
  const renderSlider = () => {
    let selectionDelta = state.endSelection - state.startSelection;

    // -6px because of the 3px width of each grabby.
    sliderViewport.style.width = state.nsToPx(selectionDelta) - 6 + "px";
    leftGrabby.style.marginLeft = state.nsToPx(state.startSelection) + "px";
    rightGrabby.style.rightMargin = state.nsToPx(state.maxTime - state.endSelection) + "px";

    renderSliderTicks();
  };

  // Render the ticks along the slider overview.
  const renderSliderTicks = oncePerWindowWidth(() => {
    let oldTicks = Array.from(window.document.querySelectorAll(".slider-tick"));
    for (let tick of oldTicks) {
      tick.remove();
    }

    let increment = selectIncrement(state.maxTime, 20);
    let px = state.nsToPx(increment);
    let ms = 0;
    for (let i = 0; i < state.windowWidth; i += px) {
      let tick = window.document.createElement("div");
      tick.className = "slider-tick";
      tick.textContent = ms + " ms";
      tick.style.left = i + "px";
      window.document.body.appendChild(tick);
      ms += increment / 1000000;
    }
  });

  // Render the individual traces.
  const renderTraces = () => {
    renderTracesTicks();

    let tracesToRender = state.getTracesInSelection();

    // Ensure that we have exactly enough trace row elements. If we have more
    // elements than traces we are going to render, then remove some. If we have
    // fewer elements than traces we are going to render, then add some.
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
  const renderTracesTicks = () => {
    let oldTicks = Array.from(tracesContainer.querySelectorAll(".traces-tick"));
    for (let tick of oldTicks) {
      tick.remove();
    }

    let selectionDelta = state.endSelection - state.startSelection;
    let increment = selectIncrement(selectionDelta, 10);
    let px = state.nsToPx(increment);
    let offset = state.startSelection % increment;
    let time = state.startSelection - offset + increment;

    while (time < state.endSelection) {
      let tick = document.createElement("div");
      tick.className = "traces-tick";
      tick.textContent = Math.round(time / 1000000) + " ms";
      tick.style.left = state.nsToSelectionPx(time - state.startSelection) + "px";
      tracesContainer.appendChild(tick);

      time += increment;
    }
  };

  // Create the DOM structure for an individual trace.
  const makeTraceTemplate = () => {
    let outer = window.document.createElement("div");
    outer.className = "outer";

    let inner = window.document.createElement("div");
    inner.className = "inner";

    let tooltip = window.document.createElement("div");
    tooltip.className = "tooltip";

    let header = window.document.createElement("h3");
    header.className = "header";
    tooltip.appendChild(header);

    let duration = window.document.createElement("h4");
    duration.className = "duration";
    tooltip.appendChild(duration);

    let pairs = window.document.createElement("dl");

    let timeStartLabel = window.document.createElement("dt");
    timeStartLabel.textContent = "Start:"
    pairs.appendChild(timeStartLabel);

    let timeStartValue = window.document.createElement("dd");
    timeStartValue.className = "start";
    pairs.appendChild(timeStartValue);

    let timeEndLabel = window.document.createElement("dt");
    timeEndLabel.textContent = "End:"
    pairs.appendChild(timeEndLabel);

    let timeEndValue = window.document.createElement("dd");
    timeEndValue.className = "end";
    pairs.appendChild(timeEndValue);

    let urlLabel = window.document.createElement("dt");
    urlLabel.textContent = "URL:";
    pairs.appendChild(urlLabel);

    let urlValue = window.document.createElement("dd");
    urlValue.className = "url";
    pairs.appendChild(urlValue);

    let iframeLabel = window.document.createElement("dt");
    iframeLabel.textContent = "iframe?";
    pairs.appendChild(iframeLabel);

    let iframeValue = window.document.createElement("dd");
    iframeValue.className = "iframe";
    pairs.appendChild(iframeValue);

    let incrementalLabel = window.document.createElement("dt");
    incrementalLabel.textContent = "Incremental?";
    pairs.appendChild(incrementalLabel);

    let incrementalValue = window.document.createElement("dd");
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
  const renderTrace = (trace, elem) => {
    let inner = elem.querySelector(".inner");
    inner.style.width = state.nsToSelectionPx(trace.endTime - trace.startTime) + "px";
    inner.style.marginLeft = state.nsToSelectionPx(trace.startTime - state.startSelection) + "px";

    let category = trace.category;
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

}(typeof exports === "object" ? exports : window,
  typeof window === "object" ? window : null));
