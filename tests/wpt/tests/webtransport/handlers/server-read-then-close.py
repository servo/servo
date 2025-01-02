def session_established(session):
    stream_id = session.create_bidirectional_stream()


def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    session.close(None)
