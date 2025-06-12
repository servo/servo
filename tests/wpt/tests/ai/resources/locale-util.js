const valid_language_tags = {
  en: [
    'en-Latn',
    'en-Latn-GB',
    'en-GB',
    'en-fonipa-scouse',
    'en-Latn-fonipa-scouse',
    'en-Latn-GB-fonipa-scouse',
    'en-Latn-x-this-is-a-private-use-extensio-n',
    'EN',
    'en-lATN',
    'EN-lATN-gb',
    'EN-gb',
    'EN-scouse-fonipa',
    'EN-lATN-scouse-fonipa',
    'EN-lATN-gb-scouse-fonipa',
  ],
  es: [
    'es-419',
    'es-ES',
    'es-ES-1979',
  ],
};

const invalid_language_tags = [
  'e',
  'Latn',
  'enLatnGBfonipa',
  '11',
  'en_Latn',
  'en-Lat',
  'en-A999',
];

function assert_is_canonical(language_tag) {
  const locale = new Intl.Locale(language_tag);
  assert_equals(locale.toString(), language_tag);
}

function assert_is_variation(variation_language_tag, expected_language_tag) {
  const variation_locale = new Intl.Locale(variation_language_tag);
  const expected_locale = new Intl.Locale(expected_language_tag);
  assert_equals(variation_locale.language, expected_locale.language);
}

function assert_availability_consistent(
    language_subtag_availability, variation_availability) {
  if (variation_availability == 'unavailable') {
    // If the language subtag is not available then no variation of it should
    // be available.
    assert_equals(language_subtag_availability, 'unavailable');
  } else {
    // If the language subtag is available, then it definitely shouldn't be
    // unavailable since whatever backing it has could support any variation of
    // it. A variation could have a different availability if a more specific
    // backing is required.
    assert_in_array(
        language_subtag_availability,
        ['downloadable', 'downloading', 'available']);
  }
}
