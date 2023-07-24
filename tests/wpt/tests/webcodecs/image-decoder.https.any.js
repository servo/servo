// META: global=window,dedicatedworker
// META: script=/webcodecs/image-decoder-utils.js

function testFourColorsDecode(filename, mimeType, options = {}) {
  var decoder = null;
  return ImageDecoder.isTypeSupported(mimeType).then(support => {
    assert_implements_optional(
        support, 'Optional codec ' + mimeType + ' not supported.');
    return fetch(filename).then(response => {
      return testFourColorsDecodeBuffer(response.body, mimeType, options);
    });
  });
}

// Note: Requiring all data to do YUV decoding is a Chromium limitation, other
// implementations may support YUV decode with partial ReadableStream data.
function testFourColorsYuvDecode(filename, mimeType, options = {}) {
  var decoder = null;
  return ImageDecoder.isTypeSupported(mimeType).then(support => {
    assert_implements_optional(
        support, 'Optional codec ' + mimeType + ' not supported.');
    return fetch(filename).then(response => {
      return response.arrayBuffer().then(buffer => {
        return testFourColorsDecodeBuffer(buffer, mimeType, options);
      });
    });
  });
}

promise_test(t => {
  return testFourColorsDecode('four-colors.jpg', 'image/jpeg');
}, 'Test JPEG image decoding.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(1);
}, 'Test JPEG w/ EXIF orientation top-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(2);
}, 'Test JPEG w/ EXIF orientation top-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(3);
}, 'Test JPEG w/ EXIF orientation bottom-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(4);
}, 'Test JPEG w/ EXIF orientation bottom-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(5);
}, 'Test JPEG w/ EXIF orientation left-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(6);
}, 'Test JPEG w/ EXIF orientation right-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(7);
}, 'Test JPEG w/ EXIF orientation right-bottom.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(8);
}, 'Test JPEG w/ EXIF orientation left-bottom.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(1, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation top-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(2, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation top-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(3, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation bottom-right.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(4, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation bottom-left.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(5, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation left-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(6, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation right-top.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(7, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation right-bottom.');

promise_test(t => {
  return testFourColorDecodeWithExifOrientation(8, null, /*useYuv=*/ true);
}, 'Test 4:2:0 JPEG w/ EXIF orientation left-bottom.');


promise_test(t => {
  return testFourColorsDecode('four-colors.png', 'image/png');
}, 'Test PNG image decoding.');

promise_test(t => {
  return testFourColorsDecode('four-colors.avif', 'image/avif');
}, 'Test AVIF image decoding.');

promise_test(t => {
  return testFourColorsDecode(
      'four-colors-full-range-bt2020-pq-444-10bpc.avif', 'image/avif',
      { tolerance: 3 });
}, 'Test high bit depth HDR AVIF image decoding.');

promise_test(t => {
  return testFourColorsDecode(
      'four-colors-flip.avif', 'image/avif', {preferAnimation: false});
}, 'Test multi-track AVIF image decoding w/ preferAnimation=false.');

promise_test(t => {
  return testFourColorsDecode(
      'four-colors-flip.avif', 'image/avif', {preferAnimation: true});
}, 'Test multi-track AVIF image decoding w/ preferAnimation=true.');

promise_test(t => {
  return testFourColorsDecode('four-colors.webp', 'image/webp');
}, 'Test WEBP image decoding.');

promise_test(t => {
  return testFourColorsDecode('four-colors.gif', 'image/gif');
}, 'Test GIF image decoding.');

promise_test(t => {
  return testFourColorsYuvDecode(
      'four-colors-limited-range-420-8bpc.jpg', 'image/jpeg',
      {yuvFormat: 'I420', tolerance: 3});
}, 'Test JPEG image YUV 4:2:0 decoding.');

promise_test(t => {
  return testFourColorsYuvDecode(
      'four-colors-limited-range-420-8bpc.avif', 'image/avif',
      {yuvFormat: 'I420', tolerance: 3});
}, 'Test AVIF image YUV 4:2:0 decoding.');

promise_test(t => {
  return testFourColorsYuvDecode(
      'four-colors-limited-range-422-8bpc.avif', 'image/avif',
      {yuvFormat: 'I422', tolerance: 3});
}, 'Test AVIF image YUV 4:2:2 decoding.');

promise_test(t => {
  return testFourColorsYuvDecode(
      'four-colors-limited-range-444-8bpc.avif', 'image/avif',
      {yuvFormat: 'I444', tolerance: 3});
}, 'Test AVIF image YUV 4:4:4 decoding.');

promise_test(t => {
  return testFourColorsYuvDecode(
      'four-colors-limited-range-420-8bpc.webp', 'image/webp',
      {yuvFormat: 'I420', tolerance: 3});
}, 'Test WEBP image YUV 4:2:0 decoding.');

promise_test(t => {
  return fetch('four-colors.png').then(response => {
    let decoder = new ImageDecoder({data: response.body, type: 'junk/type'});
    return promise_rejects_dom(t, 'NotSupportedError', decoder.decode());
  });
}, 'Test invalid mime type rejects decode() requests');

promise_test(t => {
  return fetch('four-colors.png').then(response => {
    let decoder = new ImageDecoder({data: response.body, type: 'junk/type'});
    return promise_rejects_dom(t, 'NotSupportedError', decoder.tracks.ready);
  });
}, 'Test invalid mime type rejects decodeMetadata() requests');

promise_test(t => {
  return ImageDecoder.isTypeSupported('image/png').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/png not supported.');
    return fetch('four-colors.png')
        .then(response => {
          return response.arrayBuffer();
        })
        .then(buffer => {
          let decoder = new ImageDecoder({data: buffer, type: 'image/png'});
          return promise_rejects_js(
              t, RangeError, decoder.decode({frameIndex: 1}));
        });
  });
}, 'Test out of range index returns RangeError');

promise_test(t => {
  var decoder;
  var p1;
  return ImageDecoder.isTypeSupported('image/png').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/png not supported.');
    return fetch('four-colors.png')
        .then(response => {
          return response.arrayBuffer();
        })
        .then(buffer => {
          decoder =
              new ImageDecoder({data: buffer.slice(0, 100), type: 'image/png'});
          return decoder.tracks.ready;
        })
        .then(_ => {
          // Queue two decodes to ensure index verification and decoding are
          // properly ordered.
          p1 = decoder.decode({frameIndex: 0});
          return promise_rejects_js(
              t, RangeError, decoder.decode({frameIndex: 1}));
        })
        .then(_ => {
          return promise_rejects_js(t, RangeError, p1);
        })
  });
}, 'Test partial decoding without a frame results in an error');

promise_test(t => {
  var decoder;
  var p1;
  return ImageDecoder.isTypeSupported('image/png').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/png not supported.');
    return fetch('four-colors.png')
        .then(response => {
          return response.arrayBuffer();
        })
        .then(buffer => {
          decoder =
              new ImageDecoder({data: buffer.slice(0, 100), type: 'image/png'});
          return decoder.completed;
        })
  });
}, 'Test completed property on fully buffered decode');

promise_test(t => {
  var decoder = null;

  return ImageDecoder.isTypeSupported('image/png').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/png not supported.');
    return fetch('four-colors.png')
        .then(response => {
          decoder = new ImageDecoder({data: response.body, type: 'image/png'});
          return decoder.tracks.ready;
        })
        .then(_ => {
          decoder.tracks.selectedTrack.selected = false;
          assert_equals(decoder.tracks.selectedIndex, -1);
          assert_equals(decoder.tracks.selectedTrack, null);
          return decoder.tracks.ready;
        })
        .then(_ => {
          return promise_rejects_dom(t, 'InvalidStateError', decoder.decode());
        })
        .then(_ => {
          decoder.tracks[0].selected = true;
          assert_equals(decoder.tracks.selectedIndex, 0);
          assert_not_equals(decoder.tracks.selected, null);
          return decoder.decode();
        })
        .then(result => {
          assert_equals(result.image.displayWidth, 320);
          assert_equals(result.image.displayHeight, 240);
        });
  });
}, 'Test decode, decodeMetadata after no track selected.');

promise_test(t => {
  var decoder = null;

  return ImageDecoder.isTypeSupported('image/avif').then(support => {
    assert_implements_optional(
        support, 'Optional codec image/avif not supported.');
    return fetch('four-colors-flip.avif')
        .then(response => {
          decoder = new ImageDecoder({
            data: response.body,
            type: 'image/avif',
            preferAnimation: false
          });
          return decoder.tracks.ready;
        })
        .then(_ => {
          assert_equals(decoder.tracks.length, 2);
          assert_false(decoder.tracks[decoder.tracks.selectedIndex].animated)
          assert_false(decoder.tracks.selectedTrack.animated);
          assert_equals(decoder.tracks.selectedTrack.frameCount, 1);
          assert_equals(decoder.tracks.selectedTrack.repetitionCount, 0);
          return decoder.decode();
        })
        .then(result => {
          assert_equals(result.image.displayWidth, 320);
          assert_equals(result.image.displayHeight, 240);
          assert_equals(result.image.timestamp, 0);

          // Swap to the the other track.
          let newIndex = (decoder.tracks.selectedIndex + 1) % 2;
          decoder.tracks[newIndex].selected = true;
          return decoder.decode()
        })
        .then(result => {
          assert_equals(result.image.displayWidth, 320);
          assert_equals(result.image.displayHeight, 240);
          assert_equals(result.image.timestamp, 0);
          assert_equals(result.image.duration, 10000);

          assert_equals(decoder.tracks.length, 2);
          assert_true(decoder.tracks[decoder.tracks.selectedIndex].animated)
          assert_true(decoder.tracks.selectedTrack.animated);
          assert_equals(decoder.tracks.selectedTrack.frameCount, 7);
          assert_equals(decoder.tracks.selectedTrack.repetitionCount, Infinity);
          return decoder.decode({frameIndex: 1});
        })
        .then(result => {
          assert_equals(result.image.timestamp, 10000);
          assert_equals(result.image.duration, 10000);
        });
  });
}, 'Test track selection in multi track image.');

class InfiniteGifSource {
  async load(repetitionCount) {
    let response = await fetch('four-colors-flip.gif');
    let buffer = await response.arrayBuffer();

    // Strip GIF trailer (0x3B) so we can continue to append frames.
    this.baseImage = new Uint8Array(buffer.slice(0, buffer.byteLength - 1));
    this.baseImage[0x23] = repetitionCount;
    this.counter = 0;
  }

  start(controller) {
    this.controller = controller;
    this.controller.enqueue(this.baseImage);
  }

  close() {
    this.controller.enqueue(new Uint8Array([0x3B]));
    this.controller.close();
  }

  addFrame() {
    const FRAME1_START = 0x26;
    const FRAME2_START = 0x553;

    if (this.counter++ % 2 == 0)
      this.controller.enqueue(this.baseImage.slice(FRAME1_START, FRAME2_START));
    else
      this.controller.enqueue(this.baseImage.slice(FRAME2_START));
  }
}

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(
      support, 'Optional codec image/gif not supported.');

  let source = new InfiniteGifSource();
  await source.load(5);

  let stream = new ReadableStream(source, {type: 'bytes'});
  let decoder = new ImageDecoder({data: stream, type: 'image/gif'});
  return decoder.tracks.ready
      .then(_ => {
        assert_equals(decoder.tracks.selectedTrack.frameCount, 2);
        assert_equals(decoder.tracks.selectedTrack.repetitionCount, 5);

        source.addFrame();
        return decoder.decode({frameIndex: 2});
      })
      .then(result => {
        assert_equals(decoder.tracks.selectedTrack.frameCount, 3);
        assert_equals(result.image.displayWidth, 320);
        assert_equals(result.image.displayHeight, 240);

        // Note: The stream has an alternating duration of 30ms, 40ms per frame.
        assert_equals(result.image.timestamp, 70000, "timestamp frame 2");
        assert_equals(result.image.duration, 30000, "duration frame 2");
        source.addFrame();
        return decoder.decode({frameIndex: 3});
      })
      .then(result => {
        assert_equals(decoder.tracks.selectedTrack.frameCount, 4);
        assert_equals(result.image.displayWidth, 320);
        assert_equals(result.image.displayHeight, 240);
        assert_equals(result.image.timestamp, 100000, "timestamp frame 3");
        assert_equals(result.image.duration, 40000, "duration frame 3");

        // Decode frame not yet available then reset before it comes in.
        let p = decoder.decode({frameIndex: 5});
        decoder.reset();
        return promise_rejects_dom(t, 'AbortError', p);
      })
      .then(_ => {
        // Ensure we can still decode earlier frames.
        assert_equals(decoder.tracks.selectedTrack.frameCount, 4);
        return decoder.decode({frameIndex: 3});
      })
      .then(result => {
        assert_equals(decoder.tracks.selectedTrack.frameCount, 4);
        assert_equals(result.image.displayWidth, 320);
        assert_equals(result.image.displayHeight, 240);
        assert_equals(result.image.timestamp, 100000, "timestamp frame 3");
        assert_equals(result.image.duration, 40000, "duration frame 3");

        // Decode frame not yet available then close before it comes in.
        let p = decoder.decode({frameIndex: 5});
        let tracks = decoder.tracks;
        let track = decoder.tracks.selectedTrack;
        decoder.close();

        assert_equals(decoder.type, '');
        assert_equals(decoder.tracks.length, 0);
        assert_equals(tracks.length, 0);
        track.selected = true;  // Should do nothing.

        // Previous decode should be aborted.
        return promise_rejects_dom(t, 'AbortError', p);
      })
      .then(_ => {
        // Ensure feeding the source after closing doesn't crash.
        assert_throws_js(TypeError, () => {
          source.addFrame();
        });
      });
}, 'Test ReadableStream of gif');

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(
      support, 'Optional codec image/gif not supported.');

  let source = new InfiniteGifSource();
  await source.load(5);

  let stream = new ReadableStream(source, {type: 'bytes'});
  let decoder = new ImageDecoder({data: stream, type: 'image/gif'});
  return decoder.tracks.ready.then(_ => {
    assert_equals(decoder.tracks.selectedTrack.frameCount, 2);
    assert_equals(decoder.tracks.selectedTrack.repetitionCount, 5);

    decoder.decode({frameIndex: 2}).then(t.unreached_func());
    decoder.decode({frameIndex: 1}).then(t.unreached_func());
    return decoder.tracks.ready;
  });
}, 'Test that decode requests are serialized.');

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(
      support, 'Optional codec image/gif not supported.');

  let source = new InfiniteGifSource();
  await source.load(5);

  let stream = new ReadableStream(source, {type: 'bytes'});
  let decoder = new ImageDecoder({data: stream, type: 'image/gif'});
  return decoder.tracks.ready.then(_ => {
    assert_equals(decoder.tracks.selectedTrack.frameCount, 2);
    assert_equals(decoder.tracks.selectedTrack.repetitionCount, 5);

    // Decode frame not yet available then change tracks before it comes in.
    let p = decoder.decode({frameIndex: 5});
    decoder.tracks.selectedTrack.selected = false;
    return promise_rejects_dom(t, 'AbortError', p);
  });
}, 'Test ReadableStream aborts promises on track change');

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(
      support, 'Optional codec image/gif not supported.');

  let source = new InfiniteGifSource();
  await source.load(5);

  let stream = new ReadableStream(source, {type: 'bytes'});
  let decoder = new ImageDecoder({data: stream, type: 'image/gif'});
  return decoder.tracks.ready.then(_ => {
    let p = decoder.completed;
    decoder.close();
    return promise_rejects_dom(t, 'AbortError', p);
  });
}, 'Test ReadableStream aborts completed on close');

promise_test(async t => {
  let support = await ImageDecoder.isTypeSupported('image/gif');
  assert_implements_optional(
      support, 'Optional codec image/gif not supported.');

  let source = new InfiniteGifSource();
  await source.load(5);

  let stream = new ReadableStream(source, {type: 'bytes'});
  let decoder = new ImageDecoder({data: stream, type: 'image/gif'});
  return decoder.tracks.ready.then(_ => {
    source.close();
    return decoder.completed;
  });
}, 'Test ReadableStream resolves completed');
