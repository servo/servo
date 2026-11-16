'use strict';

function is_webtransport_writer_test(interfaceName, memberName) {
  return interfaceName === 'WebTransportWriter' ||
    (interfaceName === 'WebTransportSendStream' && memberName === 'getWriter');
}
