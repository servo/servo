// Utilities for mixing rule testing.
// http://webaudio.github.io/web-audio-api/#channel-up-mixing-and-down-mixing


/**
 * Create an n-channel buffer, with all sample data zero except for a shifted
 * impulse. The impulse position depends on the channel index. For example, for
 * a 4-channel buffer:
 *  channel 0: 1 0 0 0 0 0 0 0
 *  channel 1: 0 1 0 0 0 0 0 0
 *  channel 2: 0 0 1 0 0 0 0 0
 *  channel 3: 0 0 0 1 0 0 0 0
 * @param  {AudioContext} context     Associated AudioContext.
 * @param  {Number} numberOfChannels  Number of channels of test buffer.
 * @param  {Number} frameLength       Buffer length in frames.
 * @return {AudioBuffer}
 */
function createShiftedImpulseBuffer(context, numberOfChannels, frameLength) {
  let shiftedImpulseBuffer =
      context.createBuffer(numberOfChannels, frameLength, context.sampleRate);
  for (let channel = 0; channel < numberOfChannels; ++channel) {
    let data = shiftedImpulseBuffer.getChannelData(channel);
    data[channel] = 1;
  }

  return shiftedImpulseBuffer;
}

/**
 * Create a string that displays the content of AudioBuffer.
 * @param  {AudioBuffer} audioBuffer  AudioBuffer object to stringify.
 * @param  {Number} frameLength       Number of frames to be printed.
 * @param  {Number} frameOffset       Starting frame position for printing.
 * @return {String}
 */
function stringifyBuffer(audioBuffer, frameLength, frameOffset) {
  frameOffset = (frameOffset || 0);

  let stringifiedBuffer = '';
  for (let channel = 0; channel < audioBuffer.numberOfChannels; ++channel) {
    let channelData = audioBuffer.getChannelData(channel);
    for (let i = 0; i < frameLength; ++i)
      stringifiedBuffer += channelData[i + frameOffset] + ' ';
    stringifiedBuffer += '\n';
  }

  return stringifiedBuffer;
}

/**
 * Compute number of channels from the connection.
 * http://webaudio.github.io/web-audio-api/#dfn-computednumberofchannels
 * @param  {String} connections         A string specifies the connection. For
 *                                      example, the string "128" means 3
 *                                      connections, having 1, 2, and 8 channels
 *                                      respectively.
 * @param  {Number} channelCount        Channel count.
 * @param  {String} channelCountMode    Channel count mode.
 * @return {Number}                     Computed number of channels.
 */
function computeNumberOfChannels(connections, channelCount, channelCountMode) {
  if (channelCountMode == 'explicit')
    return channelCount;

  // Must have at least one channel.
  let computedNumberOfChannels = 1;

  // Compute "computedNumberOfChannels" based on all the connections.
  for (let i = 0; i < connections.length; ++i) {
    let connectionNumberOfChannels = parseInt(connections[i]);
    computedNumberOfChannels =
        Math.max(computedNumberOfChannels, connectionNumberOfChannels);
  }

  if (channelCountMode == 'clamped-max')
    computedNumberOfChannels = Math.min(computedNumberOfChannels, channelCount);

  return computedNumberOfChannels;
}

/**
 * Apply up/down-mixing (in-place summing) based on 'speaker' interpretation.
 * @param  {AudioBuffer} input          Input audio buffer.
 * @param  {AudioBuffer} output         Output audio buffer.
 */
function speakersSum(input, output) {
  if (input.length != output.length) {
    throw '[mixing-rules.js] speakerSum(): buffer lengths mismatch (input: ' +
        input.length + ', output: ' + output.length + ')';
  }

  if (input.numberOfChannels === output.numberOfChannels) {
    for (let channel = 0; channel < output.numberOfChannels; ++channel) {
      let inputChannel = input.getChannelData(channel);
      let outputChannel = output.getChannelData(channel);
      for (let i = 0; i < outputChannel.length; i++)
        outputChannel[i] += inputChannel[i];
    }
  } else if (input.numberOfChannels < output.numberOfChannels) {
    processUpMix(input, output);
  } else {
    processDownMix(input, output);
  }
}

/**
 * In-place summing to |output| based on 'discrete' channel interpretation.
 * @param  {AudioBuffer} input          Input audio buffer.
 * @param  {AudioBuffer} output         Output audio buffer.
 */
function discreteSum(input, output) {
  if (input.length != output.length) {
    throw '[mixing-rules.js] speakerSum(): buffer lengths mismatch (input: ' +
        input.length + ', output: ' + output.length + ')';
  }

  let numberOfChannels =
      Math.min(input.numberOfChannels, output.numberOfChannels)

          for (let channel = 0; channel < numberOfChannels; ++channel) {
    let inputChannel = input.getChannelData(channel);
    let outputChannel = output.getChannelData(channel);
    for (let i = 0; i < outputChannel.length; i++)
      outputChannel[i] += inputChannel[i];
  }
}

/**
 * Perform up-mix by in-place summing to |output| buffer.
 * @param  {AudioBuffer} input          Input audio buffer.
 * @param  {AudioBuffer} output         Output audio buffer.
 */
function processUpMix(input, output) {
  let numberOfInputChannels = input.numberOfChannels;
  let numberOfOutputChannels = output.numberOfChannels;
  let i, length = output.length;

  // Up-mixing: 1 -> 2, 1 -> 4
  //   output.L += input
  //   output.R += input
  //   output.SL += 0 (in the case of 1 -> 4)
  //   output.SR += 0 (in the case of 1 -> 4)
  if ((numberOfInputChannels === 1 && numberOfOutputChannels === 2) ||
      (numberOfInputChannels === 1 && numberOfOutputChannels === 4)) {
    let inputChannel = input.getChannelData(0);
    let outputChannel0 = output.getChannelData(0);
    let outputChannel1 = output.getChannelData(1);
    for (i = 0; i < length; i++) {
      outputChannel0[i] += inputChannel[i];
      outputChannel1[i] += inputChannel[i];
    }

    return;
  }

  // Up-mixing: 1 -> 5.1
  //   output.L += 0
  //   output.R += 0
  //   output.C += input
  //   output.LFE += 0
  //   output.SL += 0
  //   output.SR += 0
  if (numberOfInputChannels == 1 && numberOfOutputChannels == 6) {
    let inputChannel = input.getChannelData(0);
    let outputChannel2 = output.getChannelData(2);
    for (i = 0; i < length; i++)
      outputChannel2[i] += inputChannel[i];

    return;
  }

  // Up-mixing: 2 -> 4, 2 -> 5.1
  //   output.L += input.L
  //   output.R += input.R
  //   output.C += 0 (in the case of 2 -> 5.1)
  //   output.LFE += 0 (in the case of 2 -> 5.1)
  //   output.SL += 0
  //   output.SR += 0
  if ((numberOfInputChannels === 2 && numberOfOutputChannels === 4) ||
      (numberOfInputChannels === 2 && numberOfOutputChannels === 6)) {
    let inputChannel0 = input.getChannelData(0);
    let inputChannel1 = input.getChannelData(1);
    let outputChannel0 = output.getChannelData(0);
    let outputChannel1 = output.getChannelData(1);
    for (i = 0; i < length; i++) {
      outputChannel0[i] += inputChannel0[i];
      outputChannel1[i] += inputChannel1[i];
    }

    return;
  }

  // Up-mixing: 4 -> 5.1
  //   output.L += input.L
  //   output.R += input.R
  //   output.C += 0
  //   output.LFE += 0
  //   output.SL += input.SL
  //   output.SR += input.SR
  if (numberOfInputChannels === 4 && numberOfOutputChannels === 6) {
    let inputChannel0 = input.getChannelData(0);    // input.L
    let inputChannel1 = input.getChannelData(1);    // input.R
    let inputChannel2 = input.getChannelData(2);    // input.SL
    let inputChannel3 = input.getChannelData(3);    // input.SR
    let outputChannel0 = output.getChannelData(0);  // output.L
    let outputChannel1 = output.getChannelData(1);  // output.R
    let outputChannel4 = output.getChannelData(4);  // output.SL
    let outputChannel5 = output.getChannelData(5);  // output.SR
    for (i = 0; i < length; i++) {
      outputChannel0[i] += inputChannel0[i];
      outputChannel1[i] += inputChannel1[i];
      outputChannel4[i] += inputChannel2[i];
      outputChannel5[i] += inputChannel3[i];
    }

    return;
  }

  // All other cases, fall back to the discrete sum.
  discreteSum(input, output);
}

/**
 * Perform down-mix by in-place summing to |output| buffer.
 * @param  {AudioBuffer} input          Input audio buffer.
 * @param  {AudioBuffer} output         Output audio buffer.
 */
function processDownMix(input, output) {
  let numberOfInputChannels = input.numberOfChannels;
  let numberOfOutputChannels = output.numberOfChannels;
  let i, length = output.length;

  // Down-mixing: 2 -> 1
  //   output += 0.5 * (input.L + input.R)
  if (numberOfInputChannels === 2 && numberOfOutputChannels === 1) {
    let inputChannel0 = input.getChannelData(0);  // input.L
    let inputChannel1 = input.getChannelData(1);  // input.R
    let outputChannel0 = output.getChannelData(0);
    for (i = 0; i < length; i++)
      outputChannel0[i] += 0.5 * (inputChannel0[i] + inputChannel1[i]);

    return;
  }

  // Down-mixing: 4 -> 1
  //   output += 0.25 * (input.L + input.R + input.SL + input.SR)
  if (numberOfInputChannels === 4 && numberOfOutputChannels === 1) {
    let inputChannel0 = input.getChannelData(0);  // input.L
    let inputChannel1 = input.getChannelData(1);  // input.R
    let inputChannel2 = input.getChannelData(2);  // input.SL
    let inputChannel3 = input.getChannelData(3);  // input.SR
    let outputChannel0 = output.getChannelData(0);
    for (i = 0; i < length; i++) {
      outputChannel0[i] += 0.25 *
          (inputChannel0[i] + inputChannel1[i] + inputChannel2[i] +
           inputChannel3[i]);
    }

    return;
  }

  // Down-mixing: 5.1 -> 1
  //   output += sqrt(1/2) * (input.L + input.R) + input.C
  //            + 0.5 * (input.SL + input.SR)
  if (numberOfInputChannels === 6 && numberOfOutputChannels === 1) {
    let inputChannel0 = input.getChannelData(0);  // input.L
    let inputChannel1 = input.getChannelData(1);  // input.R
    let inputChannel2 = input.getChannelData(2);  // input.C
    let inputChannel4 = input.getChannelData(4);  // input.SL
    let inputChannel5 = input.getChannelData(5);  // input.SR
    let outputChannel0 = output.getChannelData(0);
    let scaleSqrtHalf = Math.sqrt(0.5);
    for (i = 0; i < length; i++) {
      outputChannel0[i] +=
          scaleSqrtHalf * (inputChannel0[i] + inputChannel1[i]) +
          inputChannel2[i] + 0.5 * (inputChannel4[i] + inputChannel5[i]);
    }

    return;
  }

  // Down-mixing: 4 -> 2
  //   output.L += 0.5 * (input.L + input.SL)
  //   output.R += 0.5 * (input.R + input.SR)
  if (numberOfInputChannels == 4 && numberOfOutputChannels == 2) {
    let inputChannel0 = input.getChannelData(0);    // input.L
    let inputChannel1 = input.getChannelData(1);    // input.R
    let inputChannel2 = input.getChannelData(2);    // input.SL
    let inputChannel3 = input.getChannelData(3);    // input.SR
    let outputChannel0 = output.getChannelData(0);  // output.L
    let outputChannel1 = output.getChannelData(1);  // output.R
    for (i = 0; i < length; i++) {
      outputChannel0[i] += 0.5 * (inputChannel0[i] + inputChannel2[i]);
      outputChannel1[i] += 0.5 * (inputChannel1[i] + inputChannel3[i]);
    }

    return;
  }

  // Down-mixing: 5.1 -> 2
  //   output.L += input.L + sqrt(1/2) * (input.C + input.SL)
  //   output.R += input.R + sqrt(1/2) * (input.C + input.SR)
  if (numberOfInputChannels == 6 && numberOfOutputChannels == 2) {
    let inputChannel0 = input.getChannelData(0);    // input.L
    let inputChannel1 = input.getChannelData(1);    // input.R
    let inputChannel2 = input.getChannelData(2);    // input.C
    let inputChannel4 = input.getChannelData(4);    // input.SL
    let inputChannel5 = input.getChannelData(5);    // input.SR
    let outputChannel0 = output.getChannelData(0);  // output.L
    let outputChannel1 = output.getChannelData(1);  // output.R
    let scaleSqrtHalf = Math.sqrt(0.5);
    for (i = 0; i < length; i++) {
      outputChannel0[i] += inputChannel0[i] +
          scaleSqrtHalf * (inputChannel2[i] + inputChannel4[i]);
      outputChannel1[i] += inputChannel1[i] +
          scaleSqrtHalf * (inputChannel2[i] + inputChannel5[i]);
    }

    return;
  }

  // Down-mixing: 5.1 -> 4
  //   output.L += input.L + sqrt(1/2) * input.C
  //   output.R += input.R + sqrt(1/2) * input.C
  //   output.SL += input.SL
  //   output.SR += input.SR
  if (numberOfInputChannels === 6 && numberOfOutputChannels === 4) {
    let inputChannel0 = input.getChannelData(0);    // input.L
    let inputChannel1 = input.getChannelData(1);    // input.R
    let inputChannel2 = input.getChannelData(2);    // input.C
    let inputChannel4 = input.getChannelData(4);    // input.SL
    let inputChannel5 = input.getChannelData(5);    // input.SR
    let outputChannel0 = output.getChannelData(0);  // output.L
    let outputChannel1 = output.getChannelData(1);  // output.R
    let outputChannel2 = output.getChannelData(2);  // output.SL
    let outputChannel3 = output.getChannelData(3);  // output.SR
    let scaleSqrtHalf = Math.sqrt(0.5);
    for (i = 0; i < length; i++) {
      outputChannel0[i] += inputChannel0[i] + scaleSqrtHalf * inputChannel2[i];
      outputChannel1[i] += inputChannel1[i] + scaleSqrtHalf * inputChannel2[i];
      outputChannel2[i] += inputChannel4[i];
      outputChannel3[i] += inputChannel5[i];
    }

    return;
  }

  // All other cases, fall back to the discrete sum.
  discreteSum(input, output);
}
