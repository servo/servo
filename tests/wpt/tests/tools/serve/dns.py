from __future__ import annotations

import logging
import threading
from copy import copy
from itertools import product
from typing import TYPE_CHECKING

from dnslib import QTYPE, RCODE, RR, A
from dnslib.label import DNSLabel
from dnslib.server import BaseResolver, DNSLogger, DNSServer

if TYPE_CHECKING:
    from typing import Any, Callable

    from dnslib.dns import DNSRecord
    from dnslib.server import DNSHandler
    from wptserve.config import Config

logger = logging.getLogger()


class Resolver(BaseResolver):  # type: ignore[misc]
    def __init__(
        self,
        allowed_hosts: set[str],
        destination: str,
        unknown_labels: set[str] | None = None,
        ttl: int = 3600,
    ) -> None:
        super().__init__()
        self.unknown_labels = unknown_labels

        self.zone = [
            (rr.rname, QTYPE[rr.rtype], rr)
            for rr in (
                RR(
                    host if host.endswith(".") else host + ".",
                    rtype=QTYPE.A,
                    rdata=A(destination),
                    ttl=ttl,
                )
                for host in allowed_hosts
            )
        ]

    def resolve(self, request: DNSRecord, handler: DNSHandler) -> DNSRecord:
        reply = request.reply()
        qname = request.q.qname
        qtype = QTYPE[request.q.qtype]

        if self.unknown_labels:
            qlabels = {DNSLabel(label) for label in str(qname).rstrip(".").split(".")}
            if any(DNSLabel(label) in qlabels for label in self.unknown_labels):
                reply.header.rcode = RCODE.NXDOMAIN
                return reply

        has_answer = False
        for name, rtype, rr in self.zone:
            if qname.matchGlob(name):
                if qtype == rtype:
                    a = copy(rr)
                    a.rname = qname
                    reply.add_answer(a)
                    has_answer = True

        if has_answer:
            return reply

        reply.header.rcode = RCODE.NXDOMAIN
        return reply


class LoggingDNSLogger(DNSLogger):  # type: ignore[misc]
    def __init__(self, *args: Any, **kwargs: Any) -> None:
        self.logf: Callable[[str], None]
        super().__init__(*args, logf=logger.debug, **kwargs)

    def log_error(self, *args: Any, **kwargs: Any) -> None:
        old_logf = self.logf
        self.logf = logger.error
        try:
            super().log_error(*args, **kwargs)
        finally:
            self.logf = old_logf


class DNSServerDaemon:
    def __init__(
        self,
        host: str,
        port: int,
        bind_address: bool,
        config: Config,
        wildcards: int | None = None,
        **kwargs: Any,
    ) -> None:
        if wildcards == 0:
            wildcards = max(s.count(".") for s in config["all_domains_set"])

        if wildcards is not None:
            hosts = {config["browser_host"]} | set(config["alternate_hosts"].values())
            wildcard_hosts = {".".join(("*",) * i) for i in range(1, wildcards + 1)}
            resolver_hosts = hosts | {
                ".".join(x) + "." for x in product(wildcard_hosts, hosts)
            }
            resolver = Resolver(
                resolver_hosts, config["server_host"], config["not_subdomains"]
            )
        else:
            resolver = Resolver(config["domains_set"], config["server_host"])

        self.server = DNSServer(
            resolver,
            address=host if bind_address else "",
            port=port,
            logger=LoggingDNSLogger(),
        )

        self.server_thread: threading.Thread | None = None

    def start(self) -> None:
        self.started = True
        self.server_thread = threading.Thread(target=self.server.start)
        self.server_thread.setDaemon(True)  # don't hang on exit
        self.server_thread.start()

    def stop(self) -> None:
        if self.started:
            assert self.server_thread is not None
            try:
                self.server.shutdown()
                self.server.server_close()
                self.server_thread.join()
                self.server_thread = None
            except AttributeError:
                pass
            self.started = False
        self.server = None
