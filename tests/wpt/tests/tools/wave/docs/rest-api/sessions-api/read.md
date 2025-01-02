# `read session` - [Sessions API](../README.md#sessions-api)

The `read` method of the sessions API fetches the configuration of a session, including values that can not be set by the user, but are created by the server upon creation.

## HTTP Request

`GET /api/sessions/<session_token>`

## Response Payload

```json
{
  "token": "String",
  "tests": {
    "include": "Array<String>",
    "exclude": "Array<String>"
  },
  "types": "Enum['automatic', 'manual']",
  "timeouts": {
    "automatic": "Integer",
    "manual": "Integer",
    "<test_path>": "Integer"
  },
  "reference_tokens": "Array<String>",
  "user_agent": "String",
  "browser": {
    "name": "String",
    "version": "String"
  },
  "is_public": "Boolean",
  "date_created": "String",
  "labels": "Array<String>"
}
```

- **token** is the unique identifier of the session.
- **tests** specifies the tests of the session:
  - **include** specifies what tests should be selected from all available tests. Can be a path to a test file or directory.
  - **exclude** specifies what tests should be removed from the included tests. Can be a path to a test file or directory.
- **types** what types of tests should be included. Possible values:
  - **automatic** tests are tests that execute without user interaction.
  - **manual** tests are tests that require user interaction.
- **timeouts** specifies the time to wait for a test to finish in milliseconds.
  - **automatic**: Sets the default timeout for all automatic tests.
  - **manual**: Sets the default timeout for all manual tests.
  - **custom test paths**: Set the timeout for a test file or directory by putting the path with all dots removed as the key.
- **reference_tokens** specifies a set of completed sessions that is used to filter out all tests that have not passed in all those sessions from the session that is going to be created.
- **user_agent** is the user agent string of the request that created the session. The request to create the session should performed by the device under test.
- **browser** holds information about the browser, parsed from the user agent.
  - **name**: The name of the browser.
  - **version**: The version numbers of the browser.
- **is_public** defines whether or not the session is listed when fetching the list of public session using [`read public`](./read-public.md).
- **date_created**: The date the session was created in ISO 8601 format.
- **labels**: A list of the sessions labels.

## Example

**Request:**

`GET /api/sessions/47a6fa50-c331-11e9-8709-a8eaa0ecfd0e`

**Response:**

```json
{
  "token": "47a6fa50-c331-11e9-8709-a8eaa0ecfd0e",
  "tests": {
    "include": ["/apiOne", "/apiTwo/sub"],
    "exclude": ["/apiOne/specials"]
  },
  "types": ["automatic"],
  "timeouts": {
    "automatic": 70000,
    "/apiOne/example/dir": 30000,
    "/apiOne/example/filehtml": 45000
  },
  "reference_tokens": [
    "ce2dc080-c283-11e9-b4d6-e046513784c2",
    "430f47d0-c283-11e9-8776-fcbc36b81035"
  ],
  "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Ubuntu Chromium/76.0.3809.100 Chrome/76.0.3809.100 Safari/537.36",
  "browser": {
    "name": "Chromium",
    "version": "76"
  },
  "is_public": "false",
  "date_created": "2020-05-25T11:37:07",
  "labels": ["labelA", "labelB"]
}
```
