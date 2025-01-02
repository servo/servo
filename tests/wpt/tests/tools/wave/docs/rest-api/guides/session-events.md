# Sending and receiving session events

The session event endpoints allow to listen for events related to a specific 
session and to send new events to all registered listeners.

See all [REST API Guides](./README.md).

## Register for session specific events

To receive events of a session, simply perform a GET request to the desired 
sessions event endpoint. For example, if we want to receive any events that 
are related to the session with token `6fdbd1a0-c339-11e9-b775-6d49dd567772`:

```
GET /_wave/api/sessions/6fdbd1a0-c339-11e9-b775-6d49dd567772/events
```

```json
{
  "type": "status",
  "data": "paused"
}
```

As this endpoint makes use of the HTTP long polling, you will not immediately 
receive a response. The connection stays open either until an event gets 
triggered, in which case the server will respond with that events data, or 
there is no event within the timeout, which will return an empty response.

With one request only one event can be received. To get any further events, 
additional requests are necessary. To not miss any events, it is important to 
perform the next request immediately after receiving a response.

## Sending events

To create a new event, simply send a POST request containing the event data to 
the desired sessions event endpoint. For example, if you want to trigger a new 
event for a session with token `6fdbd1a0-c339-11e9-b775-6d49dd567772`:

```
POST /_wave/api/sessions/6fdbd1a0-c339-11e9-b775-6d49dd567772/events
```

```json
{
  "type": "status",
  "data": "paused"
}
```

This will cause any client, that currently has a connection open as described 
in the preceding section, to receive the specified event.
