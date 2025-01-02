# `read all` - [Tests API](../README.md#tests-api)

The `read all` method of the tests API fetches all tests available to include into a test session.

## HTTP Request

`GET /api/tests`

## Response Payload

```json
{
  "<api_name>": "Array<String>"
}
```

## Example

**Request:**

`GET /api/tests`

**Response:**

```json
{
  "apiOne": [
    "/apiOne/test/one.html",
    "/apiOne/test/two.html",
    "/apiOne/test/three.html"
  ],
  "apiTwo": [
    "/apiTwo/test/one.html",
    "/apiTwo/test/two.html",
    "/apiTWo/test/three.html"
  ],
  "apiThree": [
    "/apiThree/test/one.html",
    "/apiThree/test/two.html",
    "/apiThree/test/three.html"
  ]
}
```
