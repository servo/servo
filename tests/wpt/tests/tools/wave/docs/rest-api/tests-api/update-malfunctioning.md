# `update malfunctioning` - [Tests API](../README.md#tests-api)

The `update malfunctioning` method of the tests API sets the list of test files, that are flagged as not working properly in a specific session. It replaces the existing list with the new provided list.

## HTTP Request

`PUT /api/tests/<session_token>/malfunctioning`

## Request Payload

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

**Request:**

`PUT /api/tests/7dafeec0-c351-11e9-84c5-3d1ede2e7d2e/malfunctioning`

```json
[
  "/apiOne/test/three.html",
  "/apiOne/test/eight.html",
  "/apiThree/test/one.html"
]
```

**Request:**

`GET /api/tests/7dafeec0-c351-11e9-84c5-3d1ede2e7d2e/malfunctioning`

**Response:**

```json
[
  "/apiOne/test/three.html",
  "/apiOne/test/eight.html",
  "/apiThree/test/one.html"
]
```
