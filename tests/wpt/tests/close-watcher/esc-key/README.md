Tests in this directory are around the interaction of the Esc key specifically,
not the general concept of close requests. Ideally, all other tests would work
as-is if you changed the implementation of `sendCloseRequest()`. These tests
assume that Esc is the close request for the platform being tested.
