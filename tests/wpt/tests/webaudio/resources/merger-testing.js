// This file is for the audiochannelmerger-* layout tests.
// Requires |audio-testing.js| to work properly.

function testMergerInput(should, config) {
  let context = new OfflineAudioContext(config.numberOfChannels, 128, 44100);
  let merger = context.createChannelMerger(config.numberOfChannels);
  let source = context.createBufferSource();
  source.buffer = createConstantBuffer(context, 128, config.testBufferContent);

  // Connect the output of source into the specified input of merger.
  if (config.mergerInputIndex)
    source.connect(merger, 0, config.mergerInputIndex);
  else
    source.connect(merger);
  merger.connect(context.destination);
  source.start();

  return context.startRendering().then(function(buffer) {
    let prefix = config.testBufferContent.length + '-channel source: ';
    for (let i = 0; i < config.numberOfChannels; i++)
      should(buffer.getChannelData(i), prefix + 'Channel #' + i)
          .beConstantValueOf(config.expected[i]);
  });
}

async function testMergerInput_W3CTH(config) {
  const context = new OfflineAudioContext(config.numberOfChannels, 128, 44100);
  const merger = new ChannelMergerNode(context, {
    numberOfInputs: config.numberOfChannels,
  });
  const source = new AudioBufferSourceNode(context, {
    buffer: createConstantBuffer(context, 128, config.testBufferContent),
  });


  // Connect the output of source into the specified input of merger.
  if (config.mergerInputIndex) {
    source.connect(merger, 0, config.mergerInputIndex);
  } else {
    source.connect(merger);
  }
  merger.connect(context.destination);
  source.start();

  return context.startRendering().then((buffer) => {
    const prefix = config.testBufferContent.length + '-channel source: ';
    for (let i = 0; i < config.numberOfChannels; i++) {
      assert_array_equals(
          buffer.getChannelData(i),
          new Float32Array(buffer.length).fill(config.expected[i]),
          `${prefix}Channel #${i} should be constant value of ` +
              `${config.expected[i]}`);
    }
  });
}
