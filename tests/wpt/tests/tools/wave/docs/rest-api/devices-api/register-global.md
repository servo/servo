# `register global event listener` - [Devices API](../README.md#devices-api)

The `register global event listener` method of the devices API notifies a 
registered listener upon global device events. It uses HTTP long polling in 
send the event to this listener in real time, so upon receiving an event, the 
connection has to be reestablished by the client to receive further events.

## HTTP Request

`GET /api/devices/events`

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

See [global events](./event-types.md#global)

## Example

**Request:**

`GET /api/devices/events`

**Response:**

```json
{
    "type": "device_added",
    "data": {
        "token": "1d9f5d30-830f-11ea-8dcb-0021ccd76152",
        "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.113 Safari/537.36",
        "last_active": 1587391153295,
        "name": "Chrome 81.0.4044"
    }
}
```


