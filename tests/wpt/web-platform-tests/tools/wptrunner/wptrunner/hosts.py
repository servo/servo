from __future__ import unicode_literals


class HostsLine(object):
    def __init__(self, ip_address, canonical_hostname, aliases=None, comment=None):
        self.ip_address = ip_address
        self.canonical_hostname = canonical_hostname
        self.aliases = aliases if aliases is not None else []
        self.comment = comment
        if self.ip_address is None:
            assert self.canonical_hostname is None
            assert not self.aliases
            assert self.comment is not None

    @classmethod
    def from_string(cls, line):
        if not line.strip():
            return

        line = line.strip()

        ip_address = None
        canonical_hostname = None
        aliases = []
        comment = None

        comment_parts = line.split("#", 1)
        if len(comment_parts) > 1:
            comment = comment_parts[1]

        data = comment_parts[0].strip()

        if data:
            fields = data.split()
            if len(fields) < 2:
                raise ValueError("Invalid hosts line")

            ip_address = fields[0]
            canonical_hostname = fields[1]
            aliases = fields[2:]

        return cls(ip_address, canonical_hostname, aliases, comment)


class HostsFile(object):
    def __init__(self):
        self.data = []
        self.by_hostname = {}

    def set_host(self, host):
        if host.canonical_hostname is None:
            self.data.append(host)
        elif host.canonical_hostname in self.by_hostname:
            old_host = self.by_hostname[host.canonical_hostname]
            old_host.ip_address = host.ip_address
            old_host.aliases = host.aliases
            old_host.comment = host.comment
        else:
            self.data.append(host)
            self.by_hostname[host.canonical_hostname] = host

    @classmethod
    def from_file(cls, f):
        rv = cls()
        for line in f:
            host = HostsLine.from_string(line)
            if host is not None:
                rv.set_host(host)
        return rv

    def to_string(self):
        field_widths = [0, 0]
        for line in self.data:
            if line.ip_address is not None:
                field_widths[0] = max(field_widths[0], len(line.ip_address))
                field_widths[1] = max(field_widths[1], len(line.canonical_hostname))

        lines = []

        for host in self.data:
            line = ""
            if host.ip_address is not None:
                ip_string = host.ip_address.ljust(field_widths[0])
                hostname_str = host.canonical_hostname
                if host.aliases:
                    hostname_str = "%s %s" % (hostname_str.ljust(field_widths[1]),
                                              " ".join(host.aliases))
                line = "%s %s" % (ip_string, hostname_str)
            if host.comment:
                if line:
                    line += " "
                line += "#%s" % host.comment
            lines.append(line)

        lines.append("")

        return "\n".join(lines)

    def to_file(self, f):
        f.write(self.to_string().encode("utf8"))
