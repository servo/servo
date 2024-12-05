// Minimal VideoConfiguration that will be allowed per spec. All optional
// properties are missing.
var minimalVideoConfiguration = {
  contentType: 'video/webm; codecs="vp09.00.10.08"',
  width: 800,
  height: 600,
  bitrate: 3000,
  framerate: 24,
};

// Minimal WebRTC VideoConfiguration that will be allowed per spec. All optional
// properties are missing.
var minimalWebrtcVideoConfiguration = {
  contentType: 'video/VP9',
  width: 800,
  height: 600,
  bitrate: 3000,
  framerate: 24,
};

// Minimal AudioConfiguration that will be allowed per spec. All optional
// properties are missing.
var minimalAudioConfiguration = {
  contentType: 'audio/webm; codecs="opus"',
};

// Minimal WebRTC AudioConfiguration that will be allowed per spec. All optional
// properties are missing.
var minimalWebrtcAudioConfiguration = {
  contentType: 'audio/opus',
};

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo());
}, "Test that encodingInfo rejects if it doesn't get a configuration");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({}));
}, "Test that encodingInfo rejects if the MediaConfiguration isn't valid");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    video: minimalVideoConfiguration,
    audio: minimalAudioConfiguration,
  }));
}, "Test that encodingInfo rejects if the MediaConfiguration does not have a type");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
  }));
}, "Test that encodingInfo rejects if the configuration doesn't have an audio or video field");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: -1,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration has a negative framerate");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 0,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration has a framerate set to 0");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: Infinity,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration has a framerate set to Infinity");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'fgeoa',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType doesn't parse");

// See https://mimesniff.spec.whatwg.org/#example-valid-mime-type-string
promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm;',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType is not a valid MIME type string");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'audio/fgeoa',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType isn't of type video");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"; foo="bar"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType has more than one parameter");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; foo="bar"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType has one parameter that isn't codecs");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    },
  }));
}, "Test that encodingInfo rejects if the video configuration contentType does not imply a single media codec but has no codecs parameter");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08, vp8"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    }
  }));
}, "Test that encodingInfo rejects if the video configuration contentType has a codecs parameter that indicates multiple video codecs");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08, opus"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    }
  }));
}, "Test that encodingInfo rejects if the video configuration contentType has a codecs parameter that indicates both an audio and a video codec");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '24000/1001',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of x/y");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '24000/0',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of x/0");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '0/10001',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of 0/y");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '-24000/10001',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of -x/y");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '24000/-10001',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of x/-y");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '24000/',
    }
  }));
}, "Test that encodingInfo rejects framerate in the form of x/");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: '1/3x',
    }
  }));
}, "Test that encodingInfo rejects framerate with trailing unallowed characters");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'fgeoa' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contenType doesn't parse");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'video/fgeoa' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType isn't of type audio");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'audio/webm; codecs="opus"; foo="bar"' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType has more than one parameters");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'audio/webm; foo="bar"' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType has one parameter that isn't codecs");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'audio/webm' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType does not imply a single media codec but has no codecs parameter");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'audio/webm; codecs="vorbis, opus"' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType has a codecs parameter that indicates multiple audio codecs");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    audio: { contentType: 'audio/webm; codecs="vp09.00.10.08, opus"' },
  }));
}, "Test that encodingInfo rejects if the audio configuration contentType has a codecs parameter that indicates both an audio and a video codec");

promise_test(t => {
  return navigator.mediaCapabilities.encodingInfo({
    type: 'record',
    video: minimalVideoConfiguration,
    audio: minimalAudioConfiguration,
  }).then(ability => {
    assert_equals(typeof ability.supported, "boolean");
    assert_equals(typeof ability.smooth, "boolean");
    assert_equals(typeof ability.powerEfficient, "boolean");
  });
}, "Test that encodingInfo returns a valid MediaCapabilitiesInfo object for record type");

async_test(t => {
  var validTypes = [ 'record', 'webrtc' ];
  var invalidTypes = [ undefined, null, '', 'foobar', 'mse', 'MediaSource',
                       'file', 'media-source', ];

  var validPromises = [];
  var invalidCaught = 0;

  validTypes.forEach(type => {
    validPromises.push(navigator.mediaCapabilities.encodingInfo({
      type: type,
      video: type != "webrtc" ? minimalVideoConfiguration : minimalWebrtcVideoConfiguration,
      audio: type != "webrtc" ? minimalAudioConfiguration : minimalWebrtcAudioConfiguration,
    }));
  });

  // validTypes are tested via Promise.all(validPromises) because if one of the
  // promises fail, Promise.all() will reject. This mechanism can't be used for
  // invalid types which will be tested individually and increment invalidCaught
  // when rejected until the amount of rejection matches the expectation.
  Promise.all(validPromises).then(t.step_func(() => {
    for (var i = 0; i < invalidTypes.length; ++i) {
      navigator.mediaCapabilities.encodingInfo({
        type: invalidTypes[i],
        video: minimalVideoConfiguration,
        audio: minimalAudioConfiguration,
      }).then(t.unreached_func(), t.step_func(e => {
        assert_equals(e.name, 'TypeError');
        ++invalidCaught;
        if (invalidCaught == invalidTypes.length)
          t.done();
      }));
    }
  }), t.unreached_func('Promise.all should not reject for valid types'));
}, "Test that encodingInfo rejects if the MediaConfiguration does not have a valid type");
