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
