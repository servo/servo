# Session Event Types - [Sessions API](../README.md#sessions-api)

Session events are events that are triggered by actions related to sessions. 
The [`event`](./events.md) functions of the sessions API make use of these events.

## Status change

**Type identifier**: `status`  
**Payload**: `"<String>"`  
Possible Values: `paused`, `running`, `completed`, `aborted`  
**Description**: Triggered once the status of the session changes.

## Resume 

**Type identifier**: `resume`  
**Payload**: `"<String>"`  
Contains the token of the session to resume.  
**Description**: Triggered when a specific session is supposed to be resumed. 
This will discard the current session and continue executing the session with 
the provided token.

## Test Completed 

**Type identifier**: `test_completed`  
**Payload**: `"<String>"`  
Contains the test case that completed.  
**Description**: Triggered when the test runner received a result for a test.
