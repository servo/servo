// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

const testData = {
  src: 'h264.mp4',
  config: {
    codec: 'avc1.64000b',
    description: {offset: 9490, size: 45},
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  }
};

// Create a view of an ArrayBuffer.
function view(buffer, {offset, size}) {
  return new Uint8Array(buffer, offset, size);
}

function testSharedArrayBufferDescription(t, useView) {
  const data = testData;

  // Don't run test if the codec is not supported.
  let supported = false;
  return VideoDecoder.isConfigSupported({codec: data.config.codec})
      .catch(_ => {
        assert_implements_optional(false, data.config.codec + ' unsupported');
      })
      .then(support => {
        supported = support.supported;
        assert_implements_optional(
            supported, data.config.codec + ' unsupported');
        return fetch(data.src);
      })
      .then(response => {
        return response.arrayBuffer();
      })
      .then(buf => {
        config = {...data.config};
        if (data.config.description) {
          let desc = new SharedArrayBuffer(data.config.description.size);
          let descView = new Uint8Array(desc);
          descView.set(view(buf, data.config.description));
          config.description = useView ? descView : desc;
        }

        // Support was verified above, so the description shouldn't change
        // that.
        return VideoDecoder.isConfigSupported(config);
      })
      .then(support => {
        assert_true(support.supported);

        const decoder = new VideoDecoder(getDefaultCodecInit(t));
        decoder.configure(config);
        assert_equals(decoder.state, 'configured', 'state');
      });
}

promise_test(t => {
  return testSharedArrayBufferDescription(t, /*useView=*/ false);
}, 'Test isConfigSupported() and configure() using a SharedArrayBuffer');

promise_test(t => {
  return testSharedArrayBufferDescription(t, /*useView=*/ true);
}, 'Test isConfigSupported() and configure() using a Uint8Array(SharedArrayBuffer)');
