def stream_data_received(session, stream_id: int, data: bytes, stream_ended: bool):
    if session.stream_is_unidirectional(stream_id):
        return
    session.send_stream_data(stream_id, data)


def datagram_received(session, data: bytes):
    session.send_datagram(data)
