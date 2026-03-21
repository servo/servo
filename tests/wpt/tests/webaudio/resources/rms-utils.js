(function (window) {
  // Measure the RMS of the current audio output from `src` by creating a fresh
  // ScriptProcessorNode and capturing exactly one audioprocess callback, so no
  // samples from previous processing are included in the result.
  async function measureRMSOnceWithFreshScriptProcessor(ac, src) {
    function computeRMSFloat(data) {
      let sumSq = 0;
      for (let i = 0; i < data.length; i++) {
        const v = data[i];
        sumSq += v * v;
      }
      return Math.sqrt(sumSq / data.length);
    }

    // Use a new ScriptProcessorNode for each measurement, so that an event does
    // not contain samples from the past.
    const sp = ac.createScriptProcessor(2048, 1, 1);
    src.connect(sp);
    sp.connect(ac.destination);
    const { inputBuffer } =
      await new Promise(resolve => (sp.onaudioprocess = resolve));

    src.disconnect(sp);
    sp.disconnect();
    sp.onaudioprocess = null;

    const input = inputBuffer.getChannelData(0);
    const rms = computeRMSFloat(input);
    return rms;
  }

  // Repeatedly measures the RMS level from `src` using a fresh ScriptProcessor
  // until the measured value is exactly equal to `threshold`.
  // Resolves with the RMS value once the condition is met.
  async function waitForRmsEqualToThreshold(ac, src, threshold) {
    while (true) {
      const rms = await measureRMSOnceWithFreshScriptProcessor(ac, src);
        if (rms === threshold) {
        return rms;
      }
    }
  }

  // Repeatedly measures the RMS level from `src` using a fresh ScriptProcessor
  // until the measured value becomes greater than `threshold`.
  // Resolves with the RMS value once the signal exceeds the threshold.
  async function waitForRmsOverThreshold(ac, src, threshold) {
    while (true) {
      const rms = await measureRMSOnceWithFreshScriptProcessor(ac, src);
      if (rms > threshold) {
        return rms;
      }
    }
  }

  window.RMSUtils = {
    measureRMSOnceWithFreshScriptProcessor,
    waitForRmsEqualToThreshold,
    waitForRmsOverThreshold,
  };
})(window);
