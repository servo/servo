import json
import logging
import os

from collections import defaultdict, Mapping

import sslutils

from localpaths import repo_root

from .utils import get_port


_renamed_props = {
    "host": "browser_host",
    "bind_hostname": "bind_address",
    "external_host": "server_host",
    "host_ip": "server_host",
}


def _merge_dict(base_dict, override_dict):
    rv = base_dict.copy()
    for key, value in base_dict.iteritems():
        if key in override_dict:
            if isinstance(value, dict):
                rv[key] = _merge_dict(value, override_dict[key])
            else:
                rv[key] = override_dict[key]
    return rv


class Config(Mapping):
    """wptserve config

    Inherits from Mapping for backwards compatibility with the old dict-based config"""

    with open(os.path.join(repo_root, "config.default.json"), "rb") as _fp:
        _default = json.load(_fp)

    def __init__(self,
                 logger=None,
                 subdomains=set(),
                 not_subdomains=set(),
                 **kwargs):

        self.log_level = kwargs.get("log_level", "DEBUG")

        if logger is None:
            self._logger_name = "web-platform-tests"
        else:
            level_name = logging.getLevelName(logger.level)
            if level_name != "NOTSET":
                self.log_level = level_name
            self._logger_name = logger.name

        for k, v in self._default.iteritems():
            setattr(self, k, kwargs.pop(k, v))

        self.subdomains = subdomains
        self.not_subdomains = not_subdomains

        for k, new_k in _renamed_props.iteritems():
            if k in kwargs:
                self.logger.warning(
                    "%s in config is deprecated; use %s instead" % (
                        k,
                        new_k
                    )
                )
                setattr(self, new_k, kwargs.pop(k))

        self.override_ssl_env = kwargs.pop("override_ssl_env", None)

        if kwargs:
            raise TypeError("__init__() got unexpected keyword arguments %r" % (tuple(kwargs),))

    def __getitem__(self, k):
        try:
            return getattr(self, k)
        except AttributeError:
            raise KeyError(k)

    def __iter__(self):
        return iter([x for x in dir(self) if not x.startswith("_")])

    def __len__(self):
        return len([x for x in dir(self) if not x.startswith("_")])

    def update(self, override):
        """Load an overrides dict to override config values"""
        override = override.copy()

        for k in self._default:
            if k in override:
                self._set_override(k, override.pop(k))

        for k, new_k in _renamed_props.iteritems():
            if k in override:
                self.logger.warning(
                    "%s in config is deprecated; use %s instead" % (
                        k,
                        new_k
                    )
                )
                self._set_override(new_k, override.pop(k))

        if override:
            k = next(iter(override))
            raise KeyError("unknown config override '%s'" % k)

    def _set_override(self, k, v):
        old_v = getattr(self, k)
        if isinstance(old_v, dict):
            setattr(self, k, _merge_dict(old_v, v))
        else:
            setattr(self, k, v)

    @property
    def ports(self):
        try:
            old_ports = self._computed_ports
        except AttributeError:
            old_ports = {}

        self._computed_ports = defaultdict(list)

        for scheme, ports in self._ports.iteritems():
            for i, port in enumerate(ports):
                if scheme in ["wss", "https"] and not self.ssl_env.ssl_enabled:
                    port = None
                if port == "auto":
                    try:
                        port = old_ports[scheme][i]
                    except (KeyError, IndexError):
                        port = get_port(self.server_host)
                else:
                    port = port
                self._computed_ports[scheme].append(port)

        return self._computed_ports

    @ports.setter
    def ports(self, v):
        self._ports = v

    @property
    def doc_root(self):
        return self._doc_root if self._doc_root is not None else repo_root

    @doc_root.setter
    def doc_root(self, v):
        self._doc_root = v

    @property
    def ws_doc_root(self):
        if self._ws_doc_root is not None:
            return self._ws_doc_root
        else:
            return os.path.join(self.doc_root, "websockets", "handlers")

    @ws_doc_root.setter
    def ws_doc_root(self, v):
        self._ws_doc_root = v

    @property
    def server_host(self):
        return self._server_host if self._server_host is not None else self.browser_host

    @server_host.setter
    def server_host(self, v):
        self._server_host = v

    @property
    def domains(self):
        assert self.browser_host.encode("idna") == self.browser_host
        domains = {subdomain: (subdomain.encode("idna") + u"." + self.browser_host)
                   for subdomain in self.subdomains}
        domains[""] = self.browser_host
        return domains

    @property
    def not_domains(self):
        assert self.browser_host.encode("idna") == self.browser_host
        domains = {subdomain: (subdomain.encode("idna") + u"." + self.browser_host)
                   for subdomain in self.not_subdomains}
        return domains

    @property
    def all_domains(self):
        domains = self.domains.copy()
        domains.update(self.not_domains)
        return domains

    @property
    def ssl_env(self):
        try:
            if self.override_ssl_env is not None:
                return self.override_ssl_env
        except AttributeError:
            pass

        implementation_type = self.ssl["type"]

        try:
            cls = sslutils.environments[implementation_type]
        except KeyError:
            raise ValueError("%s is not a vaid ssl type." % implementation_type)
        kwargs = self.ssl.get(implementation_type, {}).copy()
        return cls(self.logger, **kwargs)

    @property
    def paths(self):
        return {"doc_root": self.doc_root,
                "ws_doc_root": self.ws_doc_root}

    @property
    def ssl_config(self):
        key_path, cert_path = self.ssl_env.host_cert_path(self.domains)
        return {"key_path": key_path,
                "cert_path": cert_path,
                "encrypt_after_connect": self.ssl["encrypt_after_connect"]}

    @property
    def log_level(self):
        return getattr(logging, self._log_level)

    @log_level.setter
    def log_level(self, value):
        self._log_level = value.upper()

    @property
    def logger(self):
        logger = logging.getLogger(self._logger_name)
        logger.setLevel(self.log_level)
        return logger

    def as_dict(self):
        rv = {
            "domains": list(self.domains),
            "sundomains": list(self.subdomains),
        }
        for item in self._default.iterkeys():
            rv[item] = getattr(self, item)
        return rv
