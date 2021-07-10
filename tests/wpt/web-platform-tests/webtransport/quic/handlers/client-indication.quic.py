import asyncio
import logging

from aioquic.asyncio import QuicConnectionProtocol
from aioquic.quic.events import QuicEvent
from typing import Dict


async def notify_pass(connection: QuicConnectionProtocol):
    _, writer = await connection.create_stream(is_unidirectional=True)
    writer.write(b'PASS')
    writer.write_eof()


# Verifies that the client indication sent by the client is correct. Creates
# a unidirectional stream containing "PASS" when the check finishes
# sucessfully.
def handle_client_indication(connection: QuicConnectionProtocol,
                             origin: str, query: Dict[str, str]):
    logging.log(logging.INFO, 'origin = %s, query = %s' % (origin, query))
    if 'origin' not in query or query['origin'] != origin:
        logging.log(logging.WARN, 'Client indication failure: invalid origin')
        connection.close()
        return

    loop = asyncio.get_event_loop()
    loop.create_task(notify_pass(connection))


def handle_event(connection: QuicConnectionProtocol, event: QuicEvent) -> None:
    pass
