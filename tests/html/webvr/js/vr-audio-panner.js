// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

(function (VRAudioPanner) {

  'use strict';

  // Default settings for panning. Cone parameters are experimentally
  // determined.
  var _PANNING_MODEL = 'HRTF';
  var _DISTANCE_MODEL = 'inverse';
  var _CONE_INNER_ANGLE = 60;
  var _CONE_OUTER_ANGLE = 120;
  var _CONE_OUTER_GAIN = 0.25;

  // Super-simple web audio version detection.
  var _LEGACY_WEBAUDIO = window.hasOwnProperty('webkitAudioContext') && !window.hasOwnProperty('AudioContext');
  if (_LEGACY_WEBAUDIO)
    console.log('[VRAudioPanner] outdated version of Web Audio API detected.');

  // Master audio context.
  var _context = _LEGACY_WEBAUDIO ? new webkitAudioContext() : new AudioContext();  
  

  /**
   * A buffer source player with HRTF panning for testing purpose.
   * @param {Object} options Default options.
   * @param {Number} options.gain Sound object gain. (0.0~1.0)
   * @param {Number} options.buffer AudioBuffer to play.
   * @param {Number} options.detune Detune parameter. (cent)
   * @param {Array} options.position x, y, z position in a array.
   */
  function TestSource (options) {

    this._src = _context.createBufferSource();
    this._out = _context.createGain();
    this._panner = _context.createPanner();
    this._analyser = _context.createAnalyser();

    this._src.connect(this._out);
    this._out.connect(this._analyser);
    this._analyser.connect(this._panner);
    this._panner.connect(_context.destination);
    
    this._src.buffer = options.buffer;
    this._src.loop = true;
    this._out.gain.value = options.gain;

    this._analyser.fftSize = 1024;
    this._analyser.smoothingTimeConstant = 0.85;
    this._lastRMSdB = 0.0;

    this._panner.panningModel = _PANNING_MODEL;
    this._panner.distanceModel = _DISTANCE_MODEL;
    this._panner.coneInnerAngle = _CONE_INNER_ANGLE;
    this._panner.coneOuterAngle = _CONE_OUTER_ANGLE;
    this._panner.coneOuterGain = _CONE_OUTER_GAIN;

    this._position = [0, 0, 0];
    this._orientation = [1, 0, 0];

    this._analyserBuffer = new Uint8Array(this._analyser.fftSize);

    if (!_LEGACY_WEBAUDIO) {
      this._src.detune.value = (options.detune || 0);
      this._analyserBuffer = new Float32Array(this._analyser.fftSize);
    }

    this.setPosition(options.position);
    this.setOrientation(options.orientation);

  };

  TestSource.prototype.start = function () {
    this._src.start(0);
  };

  TestSource.prototype.stop = function () {
    this._src.stop(0);
  };

  TestSource.prototype.getPosition = function () {
    return this._position;
  };

  TestSource.prototype.setPosition = function (position) {
    if (position) {
      this._position[0] = position[0];
      this._position[1] = position[1];
      this._position[2] = position[2];
    }

    this._panner.setPosition.apply(this._panner, this._position);
  };

  TestSource.prototype.getOrientation = function () {
    return this._orientation;
  };

  TestSource.prototype.setOrientation = function (orientation) {
    if (orientation) {
      this._orientation[0] = orientation[0];
      this._orientation[1] = orientation[1];
      this._orientation[2] = orientation[2];
    }

    this._panner.setOrientation.apply(this._panner, this._orientation);
  };

  TestSource.prototype.getCubeScale = function () {
    // Safari does not support getFloatTimeDomainData(), so fallback to the
    // naive spectral energy sum. This is relative expensive.
    if (_LEGACY_WEBAUDIO) {
      this._analyser.getByteFrequencyData(this._analyserBuffer);

      for (var k = 0, total = 0; k < this._analyserBuffer.length; ++k)
        total += this._analyserBuffer[k];
      total /= this._analyserBuffer.length;

      return (total / 256.0) * 1.5;
    }

    this._analyser.getFloatTimeDomainData(this._analyserBuffer);
    for (var i = 0, sum = 0; i < this._analyserBuffer.length; ++i)
      sum += this._analyserBuffer[i] * this._analyserBuffer[i];
    
    // Calculate RMS and convert it to DB for perceptual loudness.
    var rms = Math.sqrt(sum / this._analyserBuffer.length);
    var db = 30 + 10 / Math.LN10 * Math.log(rms <= 0 ? 0.0001 : rms);

    // Moving average with the alpha of 0.525. Experimentally determined.
    this._lastRMSdB += 0.525 * ((db < 0 ? 0 : db) - this._lastRMSdB);

    // Scaling by 1/30 is also experimentally determined.
    return this._lastRMSdB / 30.0;
  };


  // Internal helper: load a file into a buffer. (github.com/hoch/spiral)
  function _loadAudioFile(context, fileInfo, done) {
    var xhr = new XMLHttpRequest();
    xhr.open('GET', fileInfo.url);
    xhr.responseType = 'arraybuffer';

    xhr.onload = function () {
      if (xhr.status === 200) {
        context.decodeAudioData(xhr.response,
          function (buffer) {
            console.log('[VRAudioPanner] File loaded: ' + fileInfo.url);
            done(fileInfo.name, buffer);
          },
          function (message) {
            console.log('[VRAudioPanner] Decoding failure: ' + fileInfo.url + ' (' + message + ')');
            done(fileInfo.name, null);
          });
      } else {
        console.log('[VRAudioPanner] XHR Error: ' + fileInfo.url + ' (' + xhr.statusText + ')');
        done(fileInfo.name, null);
      }
    };

    xhr.onerror = function (event) {
      console.log('[VRAudioPanner] XHR Network failure: ' + fileInfo.url);
      done(fileInfo.name, null);
    };

    xhr.send();
  }


  /**
   * A wrapper/container class for multiple file loaders.
   * @param {Object} context       AudioContext
   * @param {Object} audioFileData Audio file info in the format of {name, url}
   * @param {Function} resolve       Resolution handler for promise.
   * @param {Function} reject        Rejection handler for promise.
   * @param {Function} progress      Progress event handler.
   */
  function AudioBufferManager(context, audioFileData, resolve, reject, progress) {
    this._context = context;
    this._resolve = resolve;
    this._reject = reject;
    this._progress = progress;

    this._buffers = new Map();
    this._loadingTasks = {};

    // Iterating file loading.
    for (var i = 0; i < audioFileData.length; i++) {
      var fileInfo = audioFileData[i];

      // Check for duplicates filename and quit if it happens.
      if (this._loadingTasks.hasOwnProperty(fileInfo.name)) {
        console.log('[VRAudioPanner] Duplicated filename in AudioBufferManager: ' + fileInfo.name);
        return;
      }

      // Mark it as pending (0)
      this._loadingTasks[fileInfo.name] = 0;
      _loadAudioFile(this._context, fileInfo, this._done.bind(this));
    }
  }

  AudioBufferManager.prototype._done = function (filename, buffer) {
    // Label the loading task.
    this._loadingTasks[filename] = buffer !== null ? 'loaded' : 'failed';

    // A failed task will be a null buffer.
    this._buffers.set(filename, buffer);

    this._updateProgress(filename);
  };

  AudioBufferManager.prototype._updateProgress = function (filename) {
    var numberOfFinishedTasks = 0, numberOfFailedTask = 0;
    var numberOfTasks = 0;

    for (var task in this._loadingTasks) {
      numberOfTasks++;
      if (this._loadingTasks[task] === 'loaded')
        numberOfFinishedTasks++;
      else if (this._loadingTasks[task] === 'failed')
        numberOfFailedTask++;
    }

    if (typeof this._progress === 'function')
      this._progress(filename, numberOfFinishedTasks, numberOfTasks);

    if (numberOfFinishedTasks === numberOfTasks)
      this._resolve(this._buffers);

    if (numberOfFinishedTasks + numberOfFailedTask === numberOfTasks)
      this._reject(this._buffers);
  };

  /**
   * Returns true if the web audio implementation is outdated.
   * @return {Boolean}
   */
  VRAudioPanner.isWebAudioOutdated = function () {
    return _LEGACY_WEBAUDIO;
  }

  /**
   * Static method for updating listener's position.
   * @param {Array} position Listener position in x, y, z.
   */
  VRAudioPanner.setListenerPosition = function (position) {
    _context.listener.setPosition.apply(_context.listener, position);
  };

  /**
   * Static method for updating listener's orientation.
   * @param {Array} orientation Listener orientation in x, y, z.
   * @param {Array} orientation Listener's up vector in x, y, z.
   */
  VRAudioPanner.setListenerOrientation = function (orientation, upvector) {
    _context.listener.setOrientation(
      orientation[0], orientation[1], orientation[2],
      upvector[0], upvector[1], upvector[2]);
  };

  /**
   * Load an audio file asynchronously.
   * @param {Array} dataModel Audio file info in the format of {name, url}
   * @param {Function} onprogress Callback function for reporting the progress.
   * @return {Promise} Promise.
   */
  VRAudioPanner.loadAudioFiles = function (dataModel, onprogress) {
    return new Promise(function (resolve, reject) {
      new AudioBufferManager(_context, dataModel, resolve, reject, onprogress);
    });
  };

  /**
   * Create a source player. See TestSource class for parameter description.
   * @return {TestSource}
   */
  VRAudioPanner.createTestSource = function (options) {
    return new TestSource(options);
  };

})(VRAudioPanner = {});
