# Security Policy

## Supported Versions

We are following [*CalVer*](https://calver.org) with generous backwards-compatibility guarantees.
Therefore we only support the latest version.

Put simply, you shouldn't ever be afraid to upgrade as long as you're only using our public APIs.
Whenever there is a need to break compatibility, it is announced in the changelog, and raises a `DeprecationWarning` for a year (if possible) before it's finally really broken.

> **Warning**
> The structure of the `attrs.Attribute` class is exempt from this rule.
> It *will* change in the future, but since it should be considered read-only, that shouldn't matter.
>
> However if you intend to build extensions on top of *attrs* you have to anticipate that.


## Reporting a Vulnerability

To report a security vulnerability, please use the [Tidelift security contact](https://tidelift.com/security).
Tidelift will coordinate the fix and disclosure.
