# Nanoproxy

Nanoproxy is a small (and likely noncompliant) combination of two protocols -
HTTP request proxying and mDNS advertisements. It is designed for proxying
multiple services running on a single host such as a raspberry pi within a local
network. It uses mDNS to advertise all of the configured hostnames, then proxies
the requests that come in to the upstream systems.

## Example

Imagine that you have a rasberry pi running grafana on port 3000 and prometheus
on port 5000. You want to be able to access these services from other computers
on your local network without remembering IP addresses, ports, or doing a lot of
configuration work. Nanoproxy is ideal for this use-case. Write the following
config file to `/etc/nanoconfig.toml` on your raspberry pi:

```toml
[[services]]
host_name = "prometheus"
upstream_address = "http://localhost:5000"

[[services]]
host_name = "grafana"
upstream_address = "http://localhost:3000"
```

You can then run `nanoproxy`, it will load the config file and you can navigate
to `http://prometheus.local` or `http://grafana.local` in a web browser from any
computer on your network.

## Goals

- Ease-of-use
- Low resource usage

## Non-goals of this project

- Security
  - Proxy authentication headers
  - HTTPS support
