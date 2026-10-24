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
      playing: PAUSED,
      // https://html.spec.whatwg.org/multipage/#playing-the-media-resource
      // > It is possible for a media element to have both ended playback
      // > and paused for user interaction at the same time.
      ended: ENDED,
    },
    play: {
      buffering: PLAYING,
      ended: PLAYING,
      paused: PLAYING
    }
  };

  function generateMarkup(isAudioOnly) {
    return `
      <div class="controlBar">
        <button id="play-pause-button" class="button playPauseButton" paused="true"></button>
        <div id="scrubber-stack" class="scrubberStack">
          <div class="progressBackgroundBar">
            <div class="progressStack">
              <progress id="buffer-bar" class="bufferBar" value="0" max="100"></progress>
              <progress id="progress-bar" class="progressBar" value="0" max="100"></progress>
            </div>
          </div>
          <input type="range" id="scrubber" class="scrubber" value="0" min="0" max="100" step="1">
        </div>
        <span id="position-duration-box" class="positionDurationBox hidden">
          <span id="position-text">#1</span>
          <span id="duration" class="duration"> / #2</span>
        </span>
        <button id="mute-button" class="button muteButton"></button>
        <div class="volumeStack">
          <input type="range" id="volume-level" class="volumeLevel" value="100" min="0" max="100" step="1">
        </div>
        ${isAudioOnly ? "" : '<button id="fullscreen-switch" class="button fullscreenSwitch"></button>'}
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
    // Format the duration as "h:mm:ss" or "m:ss"
    time = Math.round(time / 1000);

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
            if (this.media.hasAttribute('controls')) {
              this.initializeDom();
            } else {
              this.cleanupDom();
            }
            return;
          }
          // Here we handle element removal from DOM.
          if (mutation.type === "childList") {
            for (const node of mutation.removedNodes) {
              if (node === this.media) {
                this.cleanupDom();
                this.cleanupCompleteControls();
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

      // We also render the shadowtree for cues, which means that the
      // controls scripts run, but we don't want to show anything.
      // The above mutation observer then ensures that the controls do
      // render if at a later stage these controls should be shown.
      if (this.media.hasAttribute('controls')) {
        this.initializeDom();
      }
    }

    initializeDom() {
      // Do not reinitialize twice
      if (this.root) {
        return;
      }

      // Create root element and load markup.
      this.root = document.createElement("div");
      this.root.classList.add("controls-container");
      this.root.innerHTML = generateMarkup(this.isAudioOnly);
      this.controls.firstChild.appendChild(this.root);


      const elementNames = [
        "play-pause-button",
        "buffer-bar",
        "progress-bar",
        "scrubber",
        "position-duration-box",
        "position-text",
        "duration",
        "mute-button",
        "volume-level"
      ];

      if (!this.isAudioOnly) {
        // add "click-to-play"
        elementNames.push("fullscreen-switch");
      }

      // Import elements.
      this.elements = {};
      elementNames.forEach(id => {
        this.elements[camelCase(id)] = this.controls.getElementById(id);
      });

      // Init position duration box.
      const positionTextNode = this.elements.positionText;
      const durationSpan = this.elements.duration;
      const durationFormat = durationSpan.textContent;
      const positionFormat = positionTextNode.textContent;

      durationSpan.setAttribute("role", "none");

      Object.defineProperties(this.elements.positionDurationBox, {
        durationSpan: {
          value: durationSpan
        },
        position: {
          get: () => {
            return positionTextNode.textContent;
          },
          set: v => {
            positionTextNode.textContent = positionFormat.replace("#1", v);
          }
        },
        duration: {
          get: () => {
            return durationSpan.textContent;
          },
          set: v => {
            durationSpan.textContent = v ? durationFormat.replace("#2", v) : "";
          }
        },
        show: {
          value: (currentTime, duration) => {
            const self = this.elements.positionDurationBox;
            if (self.position != currentTime) {
              self.position = currentTime;
            }
            if (self.duration != duration) {
              self.duration = duration;
            }
            self.classList.remove("hidden");
          }
        }
      });

      // Add event listeners.
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
        "suspend"
      ];
      this.mediaEvents.forEach(event => {
        this.media.addEventListener(event, this);
      });

      this.controlEvents = [
        { el: this.elements.playPauseButton, type: "click" },
        { el: this.elements.muteButton, type: "click" },
        { el: this.elements.scrubber, type: "input" },
        { el: this.elements.volumeLevel, type: "input" }
      ];

      if (!this.isAudioOnly) {
        // this.controlEvents.push({ el: this.elements.clickToPlay, type: "click" });
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
          this.onStateChange();
        };
      }

      // Set initial state.
      this.state = this.media.paused ? PAUSED : PLAYING;
      this.onStateChange();
    }

    cleanupDom() {
      if (!this.root) {
        return;
      }
      this.mediaEvents.forEach(event => {
        this.media.removeEventListener(event, this);
      });
      this.controlEvents.forEach(({ el, type }) => {
        el.removeEventListener(type, this);
      });
      this.root.remove();
      this.root = null;
    }

    cleanupCompleteControls() {
      this.mutationObserver.disconnect();
      this.mutationObserver = null;
      this.media = null;
      this.controls = null;
    }

    // State change handler
    onStateChange() {
      this.render();
    }

    render() {
      // Error
      if (this.state == ERRORED) {
        // XXX render errored state
        return;
      }

      // Play button: show the play icon when paused or ended, the pause icon
      // otherwise
      const paused = this.state == PAUSED || this.state == ENDED;
      if (paused) {
        this.elements.playPauseButton.setAttribute("paused", "");
      } else {
        this.elements.playPauseButton.removeAttribute("paused");
      }

      // Click-to-play overlay: shown while paused, hidden while playing.
      if (this.elements.clickToPlay) {
        if (paused) {
          this.elements.clickToPlay.removeAttribute("hidden");
        } else {
          this.elements.clickToPlay.setAttribute("hidden", "");
        }
      }

      // Buffered and Playing progress 
      const duration = this.media.duration;
      const currentTime = this.media.currentTime;
      const positionPercent = duration
        ? (currentTime / duration) * 100
        : 0;
      this.elements.progressBar.value = Number.isFinite(positionPercent)
        ? positionPercent
        : 0;
      
      // TODO: Don't update the scrubber if the user is dragging it.
      this.elements.scrubber.value = Number.isFinite(positionPercent)
        ? positionPercent
        : 0;

      let bufferedEnd = 0;
      if (this.media.buffered && this.media.buffered.length > 0) {
        // Only use the last buffered range.
        bufferedEnd = this.media.buffered.end(this.media.buffered.length - 1);
      }
      const bufferedPercent = duration ? (bufferedEnd / duration) * 100 : 0;
      this.elements.bufferBar.value = Number.isFinite(bufferedPercent)
        ? bufferedPercent
        : 0;

      // Current time and duration.
      let currentTimeText = formatTime(0);
      let durationText = formatTime(0);
      if ( !isNaN(currentTime) && !isNaN(duration)) {
        currentTimeText = formatTime(currentTime * 1000);
        durationText = formatTime(duration * 1000);
      }
      this.elements.positionDurationBox.show(currentTimeText, durationText);
      const muted = this.media.muted || this.media.volume == 0;
      if (muted) {
        this.elements.muteButton.setAttribute("muted","");
      } else {
        this.elements.muteButton.removeAttribute("muted");
      }

      // Volume slider.
      const volumeLevelValue = this.media.muted
        ? 0
        : Math.round(this.media.volume * 100);
      if (this.elements.volumeLevel.value != volumeLevelValue) {
        this.elements.volumeLevel.value = volumeLevelValue;
      }
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
            case this.elements.clickToPlay:
              this.media.play();
              break;
            case this.elements.muteButton:
              this.toggleMuted();
              break;
            case this.elements.fullscreenSwitch:
              this.toggleFullscreen();
              break;
          }
          break;
        case "input":
          switch (event.currentTarget) {
            case this.elements.volumeLevel:
              this.changeVolume();
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
        case "loadeddata":
        case "loadedmetadata":
        case "progress":
        case "playing":
        case "suspend":
          this.render();
          break;
      }
    }

    /* Media actions */

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
          this.elements.fullscreenSwitch.removeAttribute("fullscreen-active");
        });
      } else {
        this.media.requestFullscreen().then(() => {
          this.elements.fullscreenSwitch.setAttribute("fullscreen-active","");
        });
      }
    }

    changeVolume() {
      const volume = parseInt(this.elements.volumeLevel.value);
      if (!isNaN(volume)) {
        this.media.volume = volume / 100;
      }
    }
  }

  new MediaControls();
})();
