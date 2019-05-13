/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

(() => {
  "use strict";

  const MARKUP = `
    <div class="controls">
      <button id="play-pause-button"></button>
      <input id="progress" type="range" value="0" min="0" max="100" step="1"></input>
      <span id="position-duration-box" class="hidden">
        #1
        <span id="duration"> / #2</span>
      </span>
      <button id="volume-switch"></button>
      <input id="volume-level" type="range" value="100" min="0" max="100" step="1"></input>
    </div>
  `;

  // States.
  const BUFFERING = "buffering";
  const ENDED = "ended";
  const ERRORED = "errored";
  const PAUSED = "paused";
  const PLAYING = "playing";

  // State transitions.
  const TRANSITIONS = {
    buffer: {
      "paused": BUFFERING,
    },
    end: {
      "playing": ENDED,
      "paused": ENDED,
    },
    error: {
      "buffering": ERRORED,
      "playing": ERRORED,
      "paused": ERRORED,
    },
    pause: {
      "buffering": PAUSED,
      "playing": PAUSED,
    },
    play: {
      "buffering": PLAYING,
      "ended": PLAYING,
      "paused": PLAYING,
    },
  };

  function camelCase(str) {
    const rdashes = /-(.)/g;
    return str.replace(rdashes, (str, p1) => {
      return p1.toUpperCase();
    });
  }

  function formatTime(time, showHours = false) {
      // Format the duration as "h:mm:ss" or "m:ss"
      time = Math.round(time / 1000);
      let hours = Math.floor(time / 3600);
      let mins  = Math.floor((time % 3600) / 60);
      let secs  = Math.floor(time % 60);
      let timeString;
      if (secs < 10) {
        secs = "0" + secs;
      }
      if (hours || showHours) {
        if (mins < 10) {
          mins = "0" + mins;
        }
        timeString = hours + ":" + mins + ":" + secs;
      } else {
        timeString = mins + ":" + secs;
      }
      return timeString;
  }

  class MediaControls {
    constructor() {
      // Get the instance of the shadow root where these controls live.
      this.controls = document.servoGetMediaControls("@@@id@@@");
      // Get the instance of the host of these controls.
      this.media = this.controls.host;

      // Create root element and load markup.
      this.root = document.createElement("div");
      this.root.classList.add("root");
      this.root.innerHTML = MARKUP;
      this.controls.appendChild(this.root);

      // Import elements.
      this.elements = {};
      [
        "duration",
        "play-pause-button",
        "position-duration-box",
        "progress",
        "volume-switch",
        "volume-level"
      ].forEach(id => {
        this.elements[camelCase(id)] = this.controls.getElementById(id);
      });

      // Init position duration box.
      const positionTextNode = Array.prototype.find.call(
        this.elements.positionDurationBox.childNodes,
        (n) => !!~n.textContent.search("#1")
      );
      const durationSpan = this.elements.duration;
      const durationFormat = durationSpan.textContent;
      const positionFormat = positionTextNode.textContent;

      durationSpan.classList.add("duration");
      durationSpan.setAttribute("role", "none");
      durationSpan.id = "durationSpan";

      Object.defineProperties(this.elements.positionDurationBox, {
        durationSpan: {
          value: durationSpan,
        },
        position: {
          get: () => {
            return positionTextNode.textContent;
          },
          set: (v) => {
            positionTextNode.textContent = positionFormat.replace("#1", v);
          },
        },
        duration: {
          get: () => {
            return durationSpan.textContent;
          },
          set: (v) => {
            durationSpan.textContent = v ? durationFormat.replace("#2", v) : "";
          },
        },
        show:{
          value: (currentTime, duration) => {
            const self = this.elements.positionDurationBox;
            if (self.position != currentTime) {
              self.position = currentTime;
            }
            if (self.duration != duration) {
              self.duration = duration;
            }
            self.classList.remove("hidden");
          },
        },
      });

      // Add event listeners.
      this.mediaEvents = [
        "play", "pause", "ended", "volumechange", "loadeddata",
        "loadstart", "timeupdate", "progress", "playing",
        "waiting", "canplay", "canplaythrough", "seeking",
        "seeked", "emptied", "loadedmetadata", "error", "suspend"];
      this.mediaEvents.forEach(event => {
        this.media.addEventListener(event, this);
      });

      this.controlEvents = [
        { el: this.elements.playPauseButton, type: "click"},
        { el: this.elements.volumeSwitch, type: "click" },
        { el: this.elements.volumeLevel, type: "input" },
      ];
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

    // State change handler
    onStateChange(from) {
      this.render(from);
    }

    render(from = this.state) {
      const isAudioOnly = this.media.localName == "audio";
      if (!isAudioOnly) {
        // XXX This should ideally use clientHeight/clientWidth,
        //     but for some reason I couldn't figure out yet,
        //     using it breaks layout.
        this.root.style.height = this.media.videoHeight;
        this.root.style.width = this.media.videoWidth;
      }

      // Error
      if (this.state == ERRORED) {
        //XXX render errored state
        return;
      }

      if (this.state != from) {
        // Play/Pause button.
        const playPauseButton = this.elements.playPauseButton;
        playPauseButton.classList.remove(from);
        playPauseButton.classList.add(this.state);
      }

      // Progress.
      const positionPercent = this.media.currentTime / this.media.duration * 100;
      if (!isNaN(positionPercent) && positionPercent != Infinity) {
        this.elements.progress.value = positionPercent;
      } else {
        this.elements.progress.value = 0;
      }

      // Current time and duration.
      let currentTime = "0:00";
      let duration = "0:00";
      if (!isNaN(this.media.currentTime) && !isNaN(this.media.duration)) {
        currentTime = formatTime(Math.round(this.media.currentTime * 1000));
        duration = formatTime(Math.round(this.media.duration * 1000));
      }
      this.elements.positionDurationBox.show(currentTime, duration);

      // Volume.
      const volumeSwitchClass = this.media.muted || this.media.volume == 0 ? "muted" : "volumeup";
      if (!this.elements.volumeSwitch.classList.contains(volumeSwitchClass)) {
        this.elements.volumeSwitch.classList = "";
        this.elements.volumeSwitch.classList.add(volumeSwitchClass);
      }
      const volumeLevelValue = Math.round((this.media.muted ? 0 : this.media.volume) * 100);
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
            case this.elements.volumeSwitch:
              this.muteUnmute();
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
        break;
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
      }
    }

    /* Media actions */

    playOrPause() {
      switch(this.state) {
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

    muteUnmute() {
      this.media.muted = !this.media.muted;
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
