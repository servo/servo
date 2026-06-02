import json

def datagram_received(session, data: bytes):
    # encode the received length into a JSON string and send back
    data_len = len(data)
    out_datagram_json = json.dumps({'length': data_len})
    out_data = out_datagram_json.encode()
    session.send_datagram(out_data)
