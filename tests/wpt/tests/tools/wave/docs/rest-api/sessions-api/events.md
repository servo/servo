# `events` - [Sessions API](../README.md#sessions-api)

Session events can be used to send messages related to a specific session for
others to receive. This can include status updates or action that running
session react on.

For possible events see [Session Event Types](./event-types.md)

## 1. `listen events`

Listen for session specific events by registering on the `events` endpoint using HTTP long polling.

### HTTP Request

```
GET /api/sessions/<token>/events
```

### Query Parameters

| Parameter    | Desciption                                                                                                                                                                                       | Default | Example        |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------- | -------------- |
| `last_event` | The number of the last received event. All events that are newer than `last_event` are returned immediately. If there are no newer events, connection stays open until a new event is triggered. | None    | `last_event=5` |

#### Response Payload

```json
[
  {
    "type": "String",
    "data": "String",
    "number": "Number"
  },
  ...
]
```

- **type**: the type of event that occurred.
- **data**: the actual payload of the event
- **number**: the number of the event

#### Example

```
GET /api/sessions/6fdbd1a0-c339-11e9-b775-6d49dd567772/events?last_event=8
```

```json
[
  {
    "type": "status",
    "data": "paused",
    "number": 9
  },
  {
    "type": "status",
    "data": "running",
    "number": 10
  },
  {
    "type": "status",
    "data": "paused",
    "number": 11
  },
  {
    "type": "status",
    "data": "running",
    "number": 12
  }
]
```

## 2. `push events`

Push session specific events for any registered listeners to receive.

### HTTP Request

```
POST /api/sessions/<token>/events
```

```json
{
  "type": "String",
  "data": "String"
}
```

- **type**: the type of event that occurred.
- **data**: the actual payload of the event

#### Example

```
POST /api/sessions/6fdbd1a0-c339-11e9-b775-6d49dd567772/events
```

```json
{
  "type": "status",
  "data": "paused"
}
```
