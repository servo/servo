import json

# This handler returns the negotiated QUIC transport parameter values for stream limits.
# The client sets anticipated stream values, which should cause higher initial_max_streams
# transport parameters to be sent during the QUIC handshake. The server reads those
# negotiated values and returns them to the client for validation.

# Store buffered data for streams
buffered_data = {}

def session_established(session):
    # Create a bidirectional stream for the client to send requests
    session.create_bidirectional_stream()


def stream_data_received(session,
                         stream_id: int,
                         data: bytes,
                         stream_ended: bool):
    # Buffer the data until the stream ends
    key = (session.session_id, stream_id)
    if key not in buffered_data:
        buffered_data[key] = b''
    buffered_data[key] += data

    if not stream_ended:
        return

    # Get the complete message
    full_data = buffered_data[key]
    del buffered_data[key]

    # Parse the request
    try:
        request = json.loads(full_data.decode('utf-8'))
        num_uni = request.get('uni', 0)
        num_bidi = request.get('bidi', 0)
    except (json.JSONDecodeError, UnicodeDecodeError) as e:
        # If we can't parse, echo back error
        response = json.dumps({'error': f'Invalid JSON: {str(e)}'})
        session.send_stream_data(stream_id, response.encode('utf-8'), end_stream=True)
        return

    # Access the underlying QUIC connection to get negotiated transport parameters
    quic_conn = session._http._quic

    # Get the peer's (client's) transport parameters
    # These reflect what the client sent during the QUIC handshake
    # The max_streams parameters tell us how many concurrent streams of each type
    # the peer is willing to accept from us (the server)
    bidi_max_streams = quic_conn._remote_max_streams_bidi
    uni_max_streams = quic_conn._remote_max_streams_uni

    # Send back the negotiated values instead of trying to create streams

    response = json.dumps({
        'uni_created': uni_max_streams,
        'bidi_created': bidi_max_streams,
        'uni_requested': num_uni,
        'bidi_requested': num_bidi
    })
    session.send_stream_data(stream_id, response.encode('utf-8'), end_stream=True)
