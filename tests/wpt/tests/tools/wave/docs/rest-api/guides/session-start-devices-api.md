# Starting sessions on a DUT using the devices API

See all [REST API Guides](./README.md).

## Connecting the DUT

To start a session on a DUT using the devices API, first register the DUT at 
the test runner.

```
POST /api/devices
```

```json
{
  "token": "fa3fb226-98ef-11ea-a21d-0021ccd76152"
}
```

Using the device token, you can listen for any events related to the device.

```
GET /api/devices/fa3fb226-98ef-11ea-a21d-0021ccd76152/events
```

Once an event occurs, the response to this call will contain the event data. 
If no event occurs until the request times out, you have to perfom another call.

```json
{
  "type": "start_session",
  "data": {
    "session_token": "98ed4b8e-98ed-11ea-9de7-0021ccd76152"
  }
}
```

Using this data you can start the session and get the URL to the next test to 
open.

## Triggering the session start

Once a device is registered and waits for events, you can use the device's 
event channel to push an event to start a session on it.

```
POST /api/devices/fa3fb226-98ef-11ea-a21d-0021ccd76152/events
```

```json
{
  "type": "start_session",
  "data": {
    "session_token": "98ed4b8e-98ed-11ea-9de7-0021ccd76152"
  }
}
```

The session related to the provided token can be a newly created one or may 
already be running.
