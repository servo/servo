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
    return `
      <div class="controls">
        <button id="play-pause-button"></button>
        <input id="progress" type="range" value="0" min="0" max="100" step="1"></input>
        <span id="position-duration-box" class="hidden">
          <span id="position-text">#1</span>
          <span id="duration"> / #2</span>
        </span>
        <button id="volume-switch"></button>
        <input id="volume-level" type="range" value="100" min="0" max="100" step="1"></input>
        ${isAudioOnly ? "" : '<button id="fullscreen-switch" class="fullscreen"></button>'}
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
    if (isNaN(time) || !isFinite(time)) return "00:00";
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

  class CustomRangeInput {
    constructor(originalInput) {
      this.originalInput = originalInput;
      this.container = document.createElement('div');
      this.container.className = 'custom-range-container';
      this.container.style.position = 'relative';
      this.container.style.height = '20px';
      this.container.style.width = '150px';
      
      this.track = document.createElement('div');
      this.track.className = 'custom-range-track';
      this.track.style.position = 'absolute';
      this.track.style.top = '50%';
      this.track.style.transform = 'translateY(-50%)';
      this.track.style.width = '100%';
      this.track.style.height = '4px';
      this.track.style.backgroundColor = '#d3d3d3';
      this.track.style.borderRadius = '2px';
      
      this.progress = document.createElement('div');
      this.progress.className = 'custom-range-progress';
      this.progress.style.position = 'absolute';
      this.progress.style.top = '50%';
      this.progress.style.transform = 'translateY(-50%)';
      this.progress.style.width = '0%';
      this.progress.style.height = '4px';
      this.progress.style.backgroundColor = '#4c8bf5';
      this.progress.style.borderRadius = '2px';
      
      this.thumb = document.createElement('div');
      this.thumb.className = 'custom-range-thumb';
      this.thumb.style.position = 'absolute';
      this.thumb.style.top = '50%';
      this.thumb.style.transform = 'translate(-50%, -50%)';
      this.thumb.style.width = '16px';
      this.thumb.style.height = '16px';
      this.thumb.style.backgroundColor = '#4c8bf5';
      this.thumb.style.borderRadius = '50%';
      this.thumb.style.cursor = 'pointer';
      this.thumb.style.zIndex = '1';

      this.originalInput.style.display = 'none';

      // Assemble component
      this.container.appendChild(this.track);
      this.container.appendChild(this.progress);
      this.container.appendChild(this.thumb);
      this.originalInput.parentNode.insertBefore(this.container, this.originalInput.nextSibling);
    
      this.updateThumbPosition();
    
      // Bind event handlers
      this.bindEvents();
    }

    updateThumbPosition() {
      const min = parseFloat(this.originalInput.min) || 0;
      const max = parseFloat(this.originalInput.max) || 100;
      const value = parseFloat(this.originalInput.value) || min;
      
      // Calculate percentage
      const percentage = ((value - min) / (max - min)) * 100;
      
      // Update thumb and progress position
      this.thumb.style.left = `${percentage}%`;
      this.progress.style.width = `${percentage}%`;
    }

    setValue(clientX) {
      const rect = this.track.getBoundingClientRect();
      const min = parseFloat(this.originalInput.min) || 0;
      const max = parseFloat(this.originalInput.max) || 100;
      const step = parseFloat(this.originalInput.step) || 1;
      
      // Calculate percentage of position within track
      let percentage = (clientX - rect.left) / rect.width;
      
      // Clamp percentage to 0-1 range
      percentage = Math.max(0, Math.min(1, percentage));
      
      // Calculate value based on percentage
      let value = min + percentage * (max - min);
      
      // Apply step if specified
      if (step > 0) {
        value = Math.round(value / step) * step;
      }
      
      // Ensure value is within min/max bounds
      value = Math.max(min, Math.min(max, value));
      
      // Update original input value
      this.originalInput.value = value;
      
      // Dispatch input and change events
      const inputEvent = new Event('input', { bubbles: true });
      const changeEvent = new Event('change', { bubbles: true });
      this.originalInput.dispatchEvent(inputEvent);
      this.originalInput.dispatchEvent(changeEvent);
      
      // Update thumb position
      this.updateThumbPosition();
    }

    bindEvents() {
      // Store bound functions so we can remove them later
      this.mouseMoveHandler = (e) => {
        if (this.isDragging) {
          this.setValue(e.clientX);
        }
      };
      
      this.mouseUpHandler = () => {
        this.isDragging = false;
      };
      
      this.touchMoveHandler = (e) => {
        if (this.isDragging) {
          this.setValue(e.touches[0].clientX);
        }
      };
      
      this.touchEndHandler = () => {
        this.isDragging = false;
      };

      // Mouse events
      this.container.addEventListener('mousedown', (e) => {
        this.isDragging = true;
        this.setValue(e.clientX);
        e.preventDefault();
      });
      
      document.addEventListener('mousemove', this.mouseMoveHandler);
      document.addEventListener('mouseup', this.mouseUpHandler);
      
      // Touch events
      this.container.addEventListener('touchstart', (e) => {
        this.isDragging = true;
        this.setValue(e.touches[0].clientX);
        e.preventDefault();
      });
      
      document.addEventListener('touchmove', this.touchMoveHandler);
      document.addEventListener('touchend', this.touchEndHandler);
    }

    destroy() {
      // Remove all event listeners
      document.removeEventListener('mousemove', this.mouseMoveHandler);
      document.removeEventListener('mouseup', this.mouseUpHandler);
      document.removeEventListener('touchmove', this.touchMoveHandler);
      document.removeEventListener('touchend', this.touchEndHandler);
      
      // Remove DOM elements
      if (this.container && this.container.parentNode) {
        this.container.parentNode.removeChild(this.container);
      }
      
      // Restore original input
      if (this.originalInput) {
        this.originalInput.style.display = '';
      }
    }
  }

  class MediaControls {
    constructor() {
      this.nonce = Date.now();
      // Get the instance of the shadow root where these controls live.
      this.controls = document.servoGetMediaControls("@@@id@@@");
      // Get the instance of the host of these controls.
      this.media = this.controls.host;

      this.mutationObserver = new MutationObserver(() => {
        // We can only get here if the `controls` attribute is removed.
        this.cleanup();
      });
      this.mutationObserver.observe(this.media, {
        attributeFilter: ["controls"]
      });

      this.isAudioOnly = this.media.localName == "audio";

      // Create root element and load markup.
      this.root = document.createElement("div");
      this.root.classList.add("root");
      this.root.innerHTML = generateMarkup(this.isAudioOnly);
      this.controls.appendChild(this.root);

      const elementNames = [
        "duration",
        "play-pause-button",
        "position-duration-box",
        "position-text",
        "progress",
        "volume-switch",
        "volume-level"
      ];

      if (!this.isAudioOnly) {
        elementNames.push("fullscreen-switch");
      }

      // Import elements.
      this.elements = {};
      elementNames.forEach(id => {
        this.elements[camelCase(id)] = this.controls.getElementById(id);
      });

      // Replace standard range inputs with custom ones
      this.customProgress = new CustomRangeInput(this.elements.progress);
      this.customVolume = new CustomRangeInput(this.elements.volumeLevel);

      // Init position duration box.
      const positionTextNode = this.elements.positionText;
      const durationSpan = this.elements.duration;
      const durationFormat = durationSpan.textContent;
      const positionFormat = positionTextNode.textContent;

      durationSpan.classList.add("duration");
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
        { el: this.elements.volumeSwitch, type: "click" },
        { el: this.elements.volumeLevel, type: "input" }
      ];

      if (!this.isAudioOnly) {
        this.controlEvents.push({ el: this.elements.fullscreenSwitch, type: "click" });
      }

      this.controlEvents.forEach(({ el, type }) => {
        el.addEventListener(type, this);
      });

      // Create state transitions.
      for (let name in TRANSITIONS) {
        if (!TRANSITIONS.hasOwnProperty(name)) {
          continue;
        }
        this[name] = () => {
          const from = this.state;

          if (!TRANSITIONS[name][from]) {
            const error = `Transition "${name}" invalid for the current state "${from}"`;
            console.error(error);
            throw new Error(error);
          }

          const to = TRANSITIONS[name][from];

          if (from == to) {
            return;
          }

          this.state = to;
          this.onStateChange(from);
        };
      }

      // Set initial state.
      this.state = this.media.paused ? PAUSED : PLAYING;
      this.onStateChange(null);
    }

    cleanup() {
      // Remove mutation observer
      this.mutationObserver.disconnect();
      
      // Remove media event listeners
      this.mediaEvents.forEach(event => {
        this.media.removeEventListener(event, this);
      });
      
      // Remove control event listeners
      this.controlEvents.forEach(({ el, type }) => {
        el.removeEventListener(type, this);
      });
      
      // Clean up custom range inputs
      if (this.customProgress) {
        this.customProgress.destroy();
      }
      if (this.customVolume) {
        this.customVolume.destroy();
      }
      
      // Remove root element
      if (this.root && this.root.parentNode) {
        this.root.parentNode.removeChild(this.root);
      }
    }

    onStateChange(from) {
      this.render(from);
    }

    render(from = this.state) {
      if (!this.isAudioOnly) {
        this.root.style.height = this.media.videoHeight;
        this.root.style.width = this.media.videoWidth;
      }

      if (this.state == ERRORED) {
        return;
      }

      if (this.state != from) {
        const playPauseButton = this.elements.playPauseButton;
        playPauseButton.classList.remove(from);
        playPauseButton.classList.add(this.state);
      }

      // Progress.
      const positionPercent =
        (this.media.currentTime / this.media.duration) * 100;
      if (Number.isFinite(positionPercent)) {
        this.elements.progress.value = positionPercent;
        this.customProgress.updateThumbPosition();
      } else {
        this.elements.progress.value = 0;
        this.customProgress.updateThumbPosition();
      }

      // Current time and duration.
      let currentTime = formatTime(0);
      let duration = formatTime(0);
      if (!isNaN(this.media.currentTime) && !isNaN(this.media.duration)) {
        currentTime = formatTime(Math.round(this.media.currentTime * 1000));
        duration = formatTime(Math.round(this.media.duration * 1000));
      }
      this.elements.positionDurationBox.show(currentTime, duration);

      // Volume.
      this.elements.volumeSwitch.className =
        this.media.muted || !this.media.volume ? "muted" : "volumeup";
      const volumeLevelValue = this.media.muted
        ? 0
        : Math.round(this.media.volume * 100);
      if (this.elements.volumeLevel.value != volumeLevelValue) {
        this.elements.volumeLevel.value = volumeLevelValue;
        this.customVolume.updateThumbPosition();
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

    onMediaEvent(event) {
      switch (event.type) {
        case "ended":
          this.end();
          break;
        case "play":
        case "pause":
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

    changeVolume() {
      const volume = parseInt(this.elements.volumeLevel.value);
      if (!isNaN(volume)) {
        this.media.volume = volume / 100;
      }
    }
  }

  new MediaControls();
})();