# `register event listener` - [Devices API](../README.md#devices-api)

The `register event listener` method of the devices API notifies a registered 
listener upon device specific events. It uses HTTP long polling in send the 
event to this listener in real time, so upon receiving an event, the 
connection has to be reestablished by the client to receive further events.

## HTTP Request

`GET /api/devices/<device_token>/events`

## Query Parameters

| Parameter        | Desciption                                                                                                                   | Example                                             |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `device_token`   | The token of the device which performed the request. (Optional)  Lets the server know the registered device is still active. | `device_token=7dafeec0-c351-11e9-84c5-3d1ede2e7d2e` |

## Response Payload

```json
{
    "type": "<String>",
    "data": "<Any>"
}
```

- **type** defines what type of event has been triggered.
- **data** contains the event specific payload.

## Event Types

### Start session

See [device specific events](./event-types.md#device-specific)

## Example

**Request:**

`GET /api/devices/1d9f5d30-830f-11ea-8dcb-0021ccd76152/events`

**Response:**

```json
{
    "type": "start_session",
    "data": {
      "session_token": "974c84e0-c35d-11e9-8f8d-47bb5bb0037d"
    }
}
```

