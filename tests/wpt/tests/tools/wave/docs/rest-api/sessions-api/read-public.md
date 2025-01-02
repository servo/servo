# `read public` - [Sessions API](../README.md#sessions-api)

The `read public` method of the sessions API fetches a list of all sessions that are publicly available. It is not possible to delete those sessions using the user interface or the REST API. Currently there is no way to change is-public-state of a session using the API.

## HTTP Request

`GET /api/sessions/public`

## Response Payload

```json
"Array<String>"
```

## Example

**Request:**

`GET /api/sessions/public`

**Response:**

```json
[
  "bb7aafa0-6a92-11e9-8ec2-04f58dad2e4f",
  "caf823e0-6a92-11e9-b732-3188d0065ebc",
  "a50c6db0-6a94-11e9-8d1b-e23fc4555885",
  "b2924d20-6a93-11e9-98b4-a11fb92a6d1c"
]
```
