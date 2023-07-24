'use strict';

test(() => {
  let recordInit = {recordType: 'w3.org:ExternalRecord'};
  const messageInit = {records: [recordInit]};
  recordInit.data = messageInit;

  assert_throws_js(TypeError, () => {
    new NDEFMessage(messageInit);
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFRecord(recordInit);
  }, 'Creating a recursive NDEFRecord throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFReadingEvent('message', {message: messageInit});
  }, 'Creating a recursive NDEFReadingEvent throws a TypeError');
}, 'NDEFRecord and NDEFMessage cycle in external records');

test(() => {
  let recordInit = {recordType: ':local'};
  const messageInit = {records: [recordInit]};
  recordInit.data = messageInit;

  const externalRecordMessageInit = {
    records: [{recordType: 'w3.org:ExternalRecord', data: messageInit}]
  };

  assert_throws_js(TypeError, () => {
    new NDEFMessage(externalRecordMessageInit);
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFRecord(externalRecordMessageInit.records[0]);
  }, 'Creating a recursive NDEFRecord throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFReadingEvent('message', {message: externalRecordMessageInit});
  }, 'Creating a recursive NDEFReadingEvent throws a TypeError');
}, 'NDEFRecord and NDEFMessage cycle in local records');

test(() => {
  let recordInit = {recordType: 'smart-poster'};
  const messageInit = {
    records: [
      // Smart poster records require an URL record. Add it here so we can be
      // sure a TypeError is being thrown because of the recursion limit, not
      // the lack of a mandatory record.
      {recordType: 'url', data: 'https://w3.org'}, recordInit
    ]
  };
  recordInit.data = messageInit;

  assert_throws_js(TypeError, () => {
    new NDEFMessage(messageInit);
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFRecord(recordInit);
  }, 'Creating a recursive NDEFRecord throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFReadingEvent('message', {message: messageInit});
  }, 'Creating a recursive NDEFReadingEvent throws a TypeError');
}, 'NDEFRecord and NDEFMessage cycle in smart poster records');

function makeSmartPosterMessageInit(innerMessageInit) {
  const innerRecords = innerMessageInit.records;
  return {
    records: [{
      recordType: 'smart-poster',
      data: {
        records:
            [{recordType: 'url', data: 'https://w3.org'}].concat(innerRecords)
      }
    }]
  };
}

// Creates an NDEFMessageInit with nested records except for the innermost
// one, which is an empty record.
function makeRecursiveMessageInit(innerRecordType, maxDepth) {
  function innerHelper(value) {
    if (++value > maxDepth) {
      return {records: [{recordType: 'empty'}]};
    }

    return {records: [{recordType: innerRecordType, data: innerHelper(value)}]};
  }

  return innerHelper(0);
}

// Maximum number of chained NDEFMessages according to the spec.
const MAX_NESTING_LEVEL = 32;

test(() => {
  // makeRecursiveMessageInit(..., N) will cause N NDEFMessages to be created
  // when it is parsed. The calls are passed to an outer NDEFMessage
  // constructor, so we end up with N+1 NDEFMessage objects. The spec allows
  // up to 32 NDEFMessages in the same chain, and we have 33 here.
  assert_throws_js(TypeError, () => {
    new NDEFMessage(
        makeRecursiveMessageInit('w3.org:ExternalRecord', MAX_NESTING_LEVEL));
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    new NDEFReadingEvent('message', {
      message:
          makeRecursiveMessageInit('w3.org:ExternalRecord', MAX_NESTING_LEVEL)
    });
  }, 'Creating a recursive NDEFReadingEvent throws a TypeError');

  // Here we call makeRecursiveMessageInit() with a smaller number than above
  // because there is a smart poster wrapping everything that also creates an
  // NDEFMessage.
  assert_throws_js(TypeError, () => {
    const innerMessageInit = makeRecursiveMessageInit(
        'w3.org:ExternalRecord', MAX_NESTING_LEVEL - 1);
    new NDEFMessage(makeSmartPosterMessageInit(innerMessageInit));
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    const innerMessageInit =
        makeRecursiveMessageInit(':local', MAX_NESTING_LEVEL - 1);
    new NDEFMessage(makeSmartPosterMessageInit(innerMessageInit));
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    const innerMessageInit = makeRecursiveMessageInit(
        'w3.org:ExternalRecord', MAX_NESTING_LEVEL - 1);
    new NDEFReadingEvent(
        'message', {message: makeSmartPosterMessageInit(innerMessageInit)});
  }, 'Creating a recursive NDEFMessage throws a TypeError');
  assert_throws_js(TypeError, () => {
    const innerMessageInit =
        makeRecursiveMessageInit(':local', MAX_NESTING_LEVEL - 1);
    new NDEFReadingEvent(
        'message', {message: makeSmartPosterMessageInit(innerMessageInit)});
  }, 'Creating a recursive NDEFMessage throws a TypeError');
}, 'Create too many nested NDEFMessages');

// See above for explanations about the counts passed to
// makeRecursiveMessageInit().
test(() => {
  new NDEFMessage(
      makeRecursiveMessageInit('w3.org:ExternalRecord', MAX_NESTING_LEVEL - 1));
  new NDEFReadingEvent('message', {
    message:
        makeRecursiveMessageInit('w3.org:ExternalRecord', MAX_NESTING_LEVEL - 1)
  });

  let innerMessageInit;

  innerMessageInit =
      makeRecursiveMessageInit('w3.org:ExternalRecord', MAX_NESTING_LEVEL - 2);
  new NDEFMessage(makeSmartPosterMessageInit(innerMessageInit));
  new NDEFReadingEvent(
      'message', {message: makeSmartPosterMessageInit(innerMessageInit)});
  innerMessageInit = makeRecursiveMessageInit(':local', MAX_NESTING_LEVEL - 2);
  new NDEFMessage(makeSmartPosterMessageInit(innerMessageInit));
  new NDEFReadingEvent(
      'message', {message: makeSmartPosterMessageInit(innerMessageInit)});
}, 'Nest maximum number of NDEFMessages')
