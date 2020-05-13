This directory contains
[QUIC](https://tools.ietf.org/html/draft-ietf-quic-transport) related tools.

# QuicTransport
[quic_transport_server.py](./quic_transport_server.py) implements a simple
[QuicTransport](https://tools.ietf.org/html/draft-vvv-webtransport-quic) server
for testing. It uses [aioquic](https://github.com/aiortc/aioquic/), and test
authors can implement custom handlers by putting python scripts in
[wpt/webtransport/quic/handlers/](../../webtransport/quic/handlers/).

## Custom Handlers
The QuicTransportServer calls functions defined in each handler script.

 - handle_client_indication is called during the client indication process.
   This function is called with three arguments:
   
   - connection: aioquic.asyncio.QuicConnectionProtocol
   - origin: str The origin of the initiator.
   - query: Dict[str, str] The dictionary of query parameters of the URL of the
            connection.

   A handler can abort the client indication process either by raising an
   exception or closing the connection.
	    
 - handle_event is called when a QuicEvent arrives.
   - connection: aioquic.asyncio.QuicConnectionProtocol
   - event: aioquic.quic.events.QuicEvent

   This function is not called until the client indication process finishes
   successfully.
