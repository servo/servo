# `read malfunctioning` - [Tests API](../README.md#tests-api)

The `read malfunctioning` method of the tests API returns a list of test files, which were flagged as not working properly in a specific session. This is useful to [add them to the exclude list](../../usage/excluding-tests.md) of further test sessions.

## HTTP Request

`GET /api/tests/<session_token>/malfunctioning`

## Response Payload

```json
"Array<String>"
```

## Example

**Request:**

`GET /api/tests/7dafeec0-c351-11e9-84c5-3d1ede2e7d2e/malfunctioning`

**Response:**

```json
[
    "/apiOne/test/one.html",
    "/apiOne/test/five.html",
    "/apiThree/test/two.html",
    "/apiThree/test/twenty.html"
]
```
