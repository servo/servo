# `create` - [Sessions API](../README.md#sessions-api)

The `create` method of the sessions API creates a new session. If provided with an configuration it creates a session accordingly. If no configuration is provided it uses default values. It returns the session token of the newly created session, which is the unique identifier of sessions. While a session has the status `PENDING` it is possible to modify the configuration using the [`update`](./update.md) method of the sessions API. As it is required to create the session from the device under test, this is really helpful, since it allows to configure the session using a second device.

## HTTP Request

`POST /api/sessions`

## Request Payload

```json
{
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
  "labels": "Array<String>",
  "type": "String"
}
```

- **tests** specifies the tests of the session:
  - **include** specifies what tests should be selected from all available tests. Can be a path to a test file or directory. Provided query parameters will be added to all matching test urls.
  - **exclude** specifies what tests should be removed from the included tests. Can be a path to a test file or directory.
- **types** what types of tests should be included. Possible values:
  - **automatic** tests are tests that execute without user interaction.
  - **manual** tests are tests that require user interaction.
- **timeouts** specifies the time to wait for a test to finish in milliseconds.
  - **automatic**: Sets the default timeout for all automatic tests.
  - **manual**: Sets the default timeout for all manual tests.
  - **custom test paths**: Set the timeout for a test file or directory by putting the path with all dots removed as the key.
- **reference_tokens** specifies a set of completed sessions that is used to filter out all tests that have not passed in all those sessions from the session that is going to be created.
- **labels** specifies the initial set of labels for the session.
- **type** specifies the session type to trigger type specific behaviour like different control pages.

### Default

```json
{
  "tests": {
    "include": ["/"],
    "exclude": []
  },
  "types": ["automatic", "manual"],
  "timeouts": {
    "automatic": 60000,
    "manual": 300000
  },
  "reference_tokens": [],
  "labels": []
}
```

## Response Payload

If successful, the token of the new session is returned.

```json
{
  "token": "String"
}
```

## Example

**Request:**

`POST /api/sessions`

```json
{
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
  "labels": ["label1", "label2", "label3"]
}
```

**Response:**

```json
{
  "token": "6fdbd1a0-c339-11e9-b775-6d49dd567772"
}
```
