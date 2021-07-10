# `events` - [Sessions API](../README.md#sessions-api)

Listen for session specific events by registering on the `events` endpoint using HTTP long polling.

## HTTP Request

`GET /api/sessions/<token>/events`

## Response Payload

```json
{
  "type": "String",
  "data": "String"
}
```

- **type**: the type of event that occurred.
- **data**: the actual payload of the event

## Example

**Request**

`GET /api/sessions/6fdbd1a0-c339-11e9-b775-6d49dd567772/events`

**Response**

```json
{
  "type": "status",
  "data": "paused"
}
```
