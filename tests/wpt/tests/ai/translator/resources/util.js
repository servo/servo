async function createTranslator(options = {}) {
  if (!options.monitor) {
    const availability = await Translator.availability(options);
    // `test_driver.bless()` below grants the user activation that `create()`
    // needs, so a 'downloadable' (or 'downloading') model is fine here; it
    // will simply be downloaded on demand.
    assert_implements_optional(
        availability !== 'unavailable',
        'Translator is not available for the given options');
  }
  await test_driver.bless();
  return await Translator.create(options);
}
