# `send event` - [Devices API](../README.md#devices-api)

The `send event` method of the devices API enables sending an event to 
listeners of specific devices events.

## HTTP Request

`POST /api/devices/<device_token>/events`

## Request Payload

```json
{
    "type": "<String>",
    "data": "<Any>"
}
```

- **type** defines what type of event has been triggered.
- **data** contains the event specific payload.

## Event Types

See [device specific events](./event-types.md#device-specific)

## Example

**Request:**

`POST /api/devices/1d9f5d30-830f-11ea-8dcb-0021ccd76152/events`

```json
{
    "type": "start_session",
    "data": {
      "session_token": "974c84e0-c35d-11e9-8f8d-47bb5bb0037d"
    }
}
```

**Response:**

`200 OK`
