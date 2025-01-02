// META: script=/resources/test-only-api.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources.js
'use strict';

contentIndexTest(async (t, index) => {
  // Exposure of the interface and method.
  assert_own_property(window, 'ContentIndex');
  assert_own_property(ContentIndex.prototype, 'add');

  assert_idl_attribute(index, 'add');
  assert_idl_attribute(index, 'delete');
  assert_idl_attribute(index, 'getAll');

}, 'The Content Index API is exposed');

contentIndexTest(async (t, index) => {
  await expectTypeError(
      index.add(createDescription({category: 'fake-category'})));

  await expectTypeError(
      index.add(createDescription({iconUrl: 'file://some-local-file.png'})));

  const isFetchingIcons = await fetchesIcons();
  if (isFetchingIcons) {
    // If the browser will try to fetch these icons we expect it to fail.
    await expectTypeError(
        index.add(createDescription({iconUrl: '/non-existent-icon.png'})));
    await expectTypeError(
        index.add(createDescription({iconUrl: '/images/broken.png'})));
  } else {
    // If the browser will not try to fetch these icons this should succeed.
    await index.add(createDescription({iconUrl: '/non-existent-icon.png'}));
    await index.add(createDescription({iconUrl: '/images/broken.png'}));
  }

  await expectTypeError(index.add(createDescription({url: 'https://other-domain.com/'})));
  await expectTypeError(index.add(createDescription({url: '/different-scope'})));

  await index.add(createDescription({}));

}, 'index.add parameters are validated.');

contentIndexTest(async (t, index) => {
  const description = createDescription({});

  // Initially there are no descriptions.
  assert_array_equals(await index.getAll(), []);

  await index.add(description);

  const descriptions = await index.getAll();
  assert_equals(descriptions.length, 1);

  assert_object_equals(descriptions[0], description);

}, 'index.getAll returns the same objects provided.');

contentIndexTest(async (t, index) => {
  const description1 = createDescription({title: 'title1'});
  const description2 = createDescription({title: 'title2'});

  await index.add(description1);
  await index.add(description2);

  // There should be one description.
  const descriptions = await index.getAll();
  assert_equals(descriptions.length, 1);

  assert_object_equals(descriptions[0], description2);

}, 'index.add with same ID overwrites existing entry.');

contentIndexTest(async (t, index) => {
  const description1 = createDescription({id: 'id1'});
  const description2 = createDescription({id: 'id2'});

  await index.add(description1);
  await index.add(description2);

  // There should be two descriptions.
  assert_equals((await index.getAll()).length, 2);

  await index.delete('id1');

  // There should be one description.
  const descriptions = await index.getAll();
  assert_equals(descriptions.length, 1);

  assert_object_equals(descriptions[0], description2);

}, 'index.delete removes entry.');

contentIndexTest(async (t, index) => {
  const descriptions = await index.getAll();
  assert_equals(descriptions.length, 0);

  await index.delete('id');

}, 'index.delete works on invalid ID.');
