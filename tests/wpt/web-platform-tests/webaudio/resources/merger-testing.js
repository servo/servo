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
