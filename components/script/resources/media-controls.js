/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

(() => {
  "use strict";

  // States.
  const BUFFERING = "buffering";
  const ENDED = "ended";
  const ERRORED = "errored";
  const PAUSED = "paused";
  const PLAYING = "playing";

  // State transitions.
  const TRANSITIONS = {
    buffer: {
      paused: BUFFERING
    },
    end: {
      playing: ENDED,
      paused: ENDED
    },
    error: {
      buffering: ERRORED,
      playing: ERRORED,
      paused: ERRORED
    },
    pause: {
      buffering: PAUSED,
      playing: PAUSED
    },
    play: {
      buffering: PLAYING,
      ended: PLAYING,
      paused: PLAYING
    }
  };

  function generateMarkup(isAudioOnly) {
    // Modern player layout:
    //   controls-wrapper: fades out entirely (gradient + controls) when idle
    //   gradient-overlay: soft blurring edge above the controls
    //   controls-row:     play/pause | time display | spacer | volume | fullscreen
    //   progress-bar:     thin seek bar at the very bottom (native range input)
    return `
      <div class="controls-wrapper">
        <div class="controls-gradient"></div>
        <div class="controls">
          <div class="controls-row">
            <button id="play-pause-button"></button>
            <span id="time-display" class="time-display">0:00 / 0:00</span>
            <div class="spacer"></div>
            <button id="volume-switch"></button>
            ${isAudioOnly ? "" : '<button id="fullscreen-switch" class="fullscreen"></button>'}
          </div>
          <div id="progress-bar" class="progress-bar">
            <div id="progress-bar-buffered" class="progress-bar-buffered"></div>
            <input id="progress" type="range" value="0" min="0" max="1000" step="1">
          </div>
        </div>
      </div>
    `;
  }

  function camelCase(str) {
    const rdashes = /-(.)/g;
    return str.replace(rdashes, (str, p1) => {
      return p1.toUpperCase();
    });
  }

  function formatTime(time, showHours = false) {
    // Time is in seconds (float). Format as "h:mm:ss" or "m:ss".
    time = Math.round(time);

    const hours = Math.floor(time / 3600);
    const mins = Math.floor((time % 3600) / 60);
    const secs = Math.floor(time % 60);

    const formattedHours =
      hours || showHours ? `${hours.toString().padStart(2, "0")}:` : "";

    return `${formattedHours}${mins
      .toString()
      .padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }

  class MediaControls {
    constructor() {
      this.nonce = Date.now();
      // Get the instance of the shadow root where these controls live.
      this.controls = document.servoGetMediaControls("@@@id@@@");
      // Get the instance of the host of these controls.
      this.media = this.controls.host;

      this.mutationObserver = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          // Here we handle the `controls` attribute removal.
          if (mutation.type === "attributes") {
            this.cleanup();
            return;
          }
          // Here we handle element removal from DOM.
          if (mutation.type === "childList") {
            for (const node of mutation.removedNodes) {
              if (node === this.media) {
                this.cleanup();
                return;
              }
            }
          }
        }
      });
      this.mutationObserver.observe(this.media, {
        attributeFilter: ["controls"]
      });
      if (this.media.parentNode) {
        this.mutationObserver.observe(this.media.parentNode, {
          childList: true
        });
      }

      this.isAudioOnly = this.media.localName == "audio";
      this.hideTimer = null;

      // Create root element and load markup.
      this.root = document.createElement("div");
      this.root.classList.add("root");
      this.root.innerHTML = generateMarkup(this.isAudioOnly);
      this.controls.appendChild(this.root);

      // Element IDs to import from the shadow DOM.
      const elementNames = [
        "play-pause-button",
        "time-display",
        "volume-switch",
        "progress-bar",
        "progress",
        "progress-bar-buffered"
      ];

      if (!this.isAudioOnly) {
        elementNames.push("fullscreen-switch");
      }

      // Import elements.
      this.elements = {};
      elementNames.forEach(id => {
        this.elements[camelCase(id)] = this.controls.getElementById(id);
      });

      // Add event listeners for media events.
      this.mediaEvents = [
        "play",
        "pause",
        "ended",
        "volumechange",
        "loadeddata",
        "loadstart",
        "timeupdate",
        "progress",
        "playing",
        "waiting",
        "canplay",
        "canplaythrough",
        "seeking",
        "seeked",
        "emptied",
        "loadedmetadata",
        "error",
        "suspend",
        "mousemove"
      ];
      this.mediaEvents.forEach(event => {
        this.media.addEventListener(event, this);
      });

      // Add event listeners for control interactions.
      this.controlEvents = [
        { el: this.elements.playPauseButton, type: "click" },
        { el: this.elements.volumeSwitch, type: "click" },
        { el: this.elements.progress, type: "input" }
      ];

      if (!this.isAudioOnly) {
        this.controlEvents.push({ el: this.elements.fullscreenSwitch, type: "click" });
      }

      this.controlEvents.forEach(({ el, type }) => {
        el.addEventListener(type, this);
      });

      // Create state transitions.
      //
      // It exposes one method per transition. i.e. this.pause(), this.play(), etc.
      // For each transition, we check that the transition is possible and call
      // the `onStateChange` handler.
      for (let name in TRANSITIONS) {
        if (!TRANSITIONS.hasOwnProperty(name)) {
          continue;
        }
        this[name] = () => {
          const from = this.state;

          // Checks if the transition is valid in the current state.
          if (!TRANSITIONS[name][from]) {
            const error = `Transition "${name}" invalid for the current state "${from}"`;
            console.error(error);
            throw new Error(error);
          }

          const to = TRANSITIONS[name][from];

          if (from == to) {
            return;
          }

          // Transition to the next state.
          this.state = to;
          this.onStateChange(from);
        };
      }

      // Set initial state.
      this.state = this.media.paused ? PAUSED : PLAYING;
      this.onStateChange(null);
    }

    cleanup() {
      this.mutationObserver.disconnect();
      this.mutationObserver = null;
      this.mediaEvents.forEach(event => {
        this.media.removeEventListener(event, this);
      });
      this.controlEvents.forEach(({ el, type }) => {
        el.removeEventListener(type, this);
      });
      this.media = null;
      this.controls = null;
    }

    // State change handler
    onStateChange(from) {
      this.render(from);
      this.manageAutohide();
    }

    render(from = this.state) {
      // Error
      if (this.state == ERRORED) {
        // XXX render errored state
        return;
      }

      if (this.state != from) {
        // Play/Pause button.
        const playPauseButton = this.elements.playPauseButton;
        playPauseButton.classList.remove(from);
        playPauseButton.classList.add(this.state);
      }

      // Progress bar (native range input, 0–1000 scale).
      const positionPermille =
        (this.media.currentTime / this.media.duration) * 1000;
      if (Number.isFinite(positionPermille)) {
        this.elements.progress.value = Math.round(positionPermille);
      } else {
        this.elements.progress.value = 0;
      }

      // Buffered progress.
      if (this.media.buffered && this.media.buffered.length > 0) {
        const bufferedEnd = this.media.buffered.end(this.media.buffered.length - 1);
        const bufferedPercent = (bufferedEnd / this.media.duration) * 100;
        if (Number.isFinite(bufferedPercent)) {
          this.elements.progressBarBuffered.style.width = bufferedPercent + "%";
        }
      }

      // Time display.
      let currentTime = formatTime(0);
      let duration = formatTime(0);
      if (!isNaN(this.media.currentTime) && !isNaN(this.media.duration)) {
        currentTime = formatTime(this.media.currentTime);
        duration = formatTime(this.media.duration);
      }
      this.elements.timeDisplay.textContent = `${currentTime} / ${duration}`;

      // Volume button.
      this.elements.volumeSwitch.className =
        this.media.muted || !this.media.volume ? "muted" : "volumeup";
    }

    handleEvent(event) {
      if (!event.isTrusted) {
        console.warn(`Drop untrusted event ${event.type}`);
        return;
      }

      if (this.mediaEvents.includes(event.type)) {
        this.onMediaEvent(event);
      } else {
        this.onControlEvent(event);
      }
    }

    onControlEvent(event) {
      switch (event.type) {
        case "click":
          switch (event.currentTarget) {
            case this.elements.playPauseButton:
              this.playOrPause();
              break;
            case this.elements.volumeSwitch:
              this.toggleMuted();
              break;
            case this.elements.fullscreenSwitch:
              this.toggleFullscreen();
              break;
          }
          break;
        case "input":
          switch (event.currentTarget) {
            case this.elements.progress:
              this.seekFromSlider(event);
              break;
          }
          break;
        default:
          throw new Error(`Unknown event ${event.type}`);
      }
    }

    // HTMLMediaElement event handler
    onMediaEvent(event) {
      switch (event.type) {
        case "ended":
          this.end();
          break;
        case "play":
        case "pause":
          // Transition to PLAYING or PAUSED state.
          this[event.type]();
          break;
        case "volumechange":
        case "timeupdate":
        case "resize":
          this.render();
          break;
        case "loadedmetadata":
          break;
        case "mousemove":
          this.showControls();
          this.resetHideTimer();
          break;
      }
    }

    /* ── Media actions ── */

    playOrPause() {
      switch (this.state) {
        case PLAYING:
          this.media.pause();
          break;
        case BUFFERING:
        case ENDED:
        case PAUSED:
          this.media.play();
          break;
        default:
          throw new Error(`Invalid state ${this.state}`);
      }
    }

    toggleMuted() {
      this.media.muted = !this.media.muted;
    }

    toggleFullscreen() {
      const { fullscreenEnabled, fullscreenElement } = document;

      const isElementFullscreen = fullscreenElement && fullscreenElement === this.media;

      if (fullscreenEnabled && isElementFullscreen) {
        document.exitFullscreen().then(() => {
          this.elements.fullscreenSwitch.classList.remove("fullscreen-active");
        });
      } else {
        this.media.requestFullscreen().then(() => {
          this.elements.fullscreenSwitch.classList.add("fullscreen-active");
        });
      }
    }

    seekFromSlider(event) {
      // Range input value is 0–1000, map to currentTime.
      const value = parseInt(event.target.value);
      if (!isNaN(value) && this.media.duration) {
        this.media.currentTime = (value / 1000) * this.media.duration;
      }
    }

    /* ── Auto-hide controls during playback ── */

    // Decide whether to show or start hiding controls based on playback state.
    manageAutohide() {
      if (this.state === PLAYING) {
        this.startHideTimer();
      } else {
        this.cancelHideTimer();
        this.showControls();
      }
    }

    // Begin countdown to hide controls (3 seconds of no mouse activity).
    startHideTimer() {
      this.cancelHideTimer();
      this.hideTimer = setTimeout(() => {
        const wrapper = this.controls.querySelector(".controls-wrapper");
        if (wrapper) {
          wrapper.classList.add("autohide");
        }
      }, 3000);
    }

    // Cancel the pending hide timer.
    cancelHideTimer() {
      if (this.hideTimer) {
        clearTimeout(this.hideTimer);
        this.hideTimer = null;
      }
    }

    // Restart the hide timer (called on mousemove during playback).
    resetHideTimer() {
      if (this.state === PLAYING) {
        this.startHideTimer();
      }
    }

    // Reveal the controls by removing the autohide class.
    showControls() {
      const wrapper = this.controls.querySelector(".controls-wrapper");
      if (wrapper) {
        wrapper.classList.remove("autohide");
      }
    }
  }

  new MediaControls();
})();
