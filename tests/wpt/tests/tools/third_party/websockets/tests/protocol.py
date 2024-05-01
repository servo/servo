from websockets.protocol import Protocol


class RecordingProtocol(Protocol):
    """
    Protocol subclass that records incoming frames.

    By interfacing with this protocol, you can check easily what the component
    being testing sends during a test.

    """

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.frames_rcvd = []

    def get_frames_rcvd(self):
        """
        Get incoming frames received up to this point.

        Calling this method clears the list. Each frame is returned only once.

        """
        frames_rcvd, self.frames_rcvd = self.frames_rcvd, []
        return frames_rcvd

    def recv_frame(self, frame):
        self.frames_rcvd.append(frame)
        super().recv_frame(frame)
