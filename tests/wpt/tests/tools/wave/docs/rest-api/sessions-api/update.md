# `update` - [Sessions API](../README.md#sessions-api)

The `update` method of the sessions API makes it possible to modify a sessions configuration while its status is `PENDING`. This can be used to configure the session on a second device, rather than on the device under test.

## HTTP Request

`PUT /api/sessions/<session_token>`

## Request Payload

The request payload is the same as in the [`create`](./sessions-api/create.md) method of the sessions API. Only keys that are an inherent part of the configuration will stay the same if not specified in the `update` payload. All others will be deleted if not included.

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
  "labels": []
}
```

**Request:**

`PUT /api/sessions/47a6fa50-c331-11e9-8709-a8eaa0ecfd0e`

```json
{
  "tests": {
    "include": ["/apiOne", "/apiThree"]
  },
  "timeouts": {
    "automatic": 60000
  },
  "reference_tokens": [
    "bb7aafa0-6a92-11e9-8ec2-04f58dad2e4f",
    "a50c6db0-6a94-11e9-8d1b-e23fc4555885"
  ],
  "labels": ["label1", "label2"]
}
```

**Response:**

`200 OK`

**Request:**

`GET /api/sessions/47a6fa50-c331-11e9-8709-a8eaa0ecfd0e`

**Response:**

```json
{
  "token": "47a6fa50-c331-11e9-8709-a8eaa0ecfd0e",
  "tests": {
    "include": ["/apiOne", "/apiThree"],
    "exclude": ["/apiOne/specials"]
  },
  "types": ["automatic"],
  "timeouts": {
    "automatic": 60000,
    "manual": 360000
  },
  "reference_tokens": [
    "bb7aafa0-6a92-11e9-8ec2-04f58dad2e4f",
    "a50c6db0-6a94-11e9-8d1b-e23fc4555885"
  ],
  "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Ubuntu Chromium/76.0.3809.100 Chrome/76.0.3809.100 Safari/537.36",
  "browser": {
    "name": "Chromium",
    "version": "76"
  },
  "is_public": "false",
  "labels": ["label1", "label2"]
}
```
