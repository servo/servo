'use strict';

// Parses a response string as JSON and asserts on failure.
function parse_json_response(response) {
  try {
    return JSON.parse(response);
  } catch (e) {
    assert_unreached(
        `Response is not valid JSON: "${response}". Error: ${e.message}`);
  }
}

const kValidResponseSchema = {
  type: 'object',
  required: ['Rating'],
  additionalProperties: false,
  properties: {
    Rating: {
      type: 'number',
      minimum: 0,
      maximum: 5,
    },
  },
};

function testResponseJsonSchema(response, t) {
  const jsonResponse = parse_json_response(response);
  assert_equals(typeof jsonResponse, 'object', 'Response should be an object');
  assert_own_property(jsonResponse, 'Rating',
                      'JSON response should have a "Rating" property.');
  assert_equals(typeof jsonResponse.Rating, 'number',
                'Rating should be a number');
  assert_greater_than_equal(jsonResponse.Rating, 0, 'Rating should be >= 0');
  assert_less_than_equal(jsonResponse.Rating, 5, 'Rating should be <= 5');
}
