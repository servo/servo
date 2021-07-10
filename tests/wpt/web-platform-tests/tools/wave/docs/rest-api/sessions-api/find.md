# `find` - [Sessions API](../README.md#sessions-api)

The `find` method of the sessions API searches for a session token using a provided token fragment, which is the beginning of a session token with at least 8 characters. Due to data protection, it is not possible to find multiple tokens using one fragment. If the server finds more than one session token, it returns none. In this case more characters need to be added to the fragment, until it matches only one session token.

## HTTP Request

`GET /api/sessions/<token_fragment>`

## Response Payload

```json
{
  "token": "String"
}
```

### Example

**Request:**

`GET /api/sessions/afd4ecb0`

**Response:**

```json
{
  "token": "afd4ecb0-c339-11e9-b66c-eca76c2bea9c"
}
```
