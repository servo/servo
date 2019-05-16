{% set bin = "/usr/local/bin" %}
{% set etc = "/etc/generic-worker" %}
{% set user = "worker" %}
{% set home = "/Users/" + user %}

GMT:
  timezone.system

{{ bin }}/generic-worker:
  file.managed:
    - name:
    - source: https://github.com/taskcluster/generic-worker/releases/download/v14.1.1/generic-worker-nativeEngine-darwin-amd64
    - source_hash: sha256=817e72972a7c077f1a829d5824e5c0e831eb6f9b254672e7427246a8dd476a59
    - mode: 755
    - makedirs: True
    - watch_in:
      - service: net.generic.worker

{{ bin }}/livelog:
  file.managed:
    - source: https://github.com/taskcluster/livelog/releases/download/v1.1.0/livelog-darwin-amd64
    - source_hash: sha256=be5d4b998b208afd802ac6ce6c4d4bbf0fb3816bb039a300626abbc999dfe163
    - mode: 755
    - makedirs: True
    - watch_in:
      - service: net.generic.worker

{{ bin }}/taskcluster-proxy:
  file.managed:
    - source: https://github.com/taskcluster/taskcluster-proxy/releases/download/v5.1.0/taskcluster-proxy-darwin-amd64
    - source_hash: sha256=3faf524b9c6b9611339510797bf1013d4274e9f03e7c4bd47e9ab5ec8813d3ae
    - mode: 755
    - makedirs: True
    - watch_in:
      - service: net.generic.worker

{{ user }} group:
  group.present:
    - name: {{ user }}

{{ user }}:
  user.present:
    - home: {{ home }}
    - gid_from_name: True

# `user.present`’s `createhome` is apparently not supported on macOS
{{ home }}:
  file.directory:
    - user: {{ user }}

{{ etc }}/config.json:
  file.serialize:
    - makedirs: True
    - group: {{ user }}
    - mode: 640
    - show_changes: False
    - formatter: json
    - dataset:
        provisionerId: proj-servo
        workerType: macos
        workerGroup: servo-macos
        workerId: {{ grains["id"] }}
        tasksDir: {{ home }}/tasks
        publicIP: {{ salt.network.ip_addrs()[0] }}
        ed25519SigningKeyLocation: {{ home }}/keypair
        clientId: {{ pillar["client_id"] }}
        accessToken: {{ pillar["access_token"] }}
        taskclusterProxyExecutable: {{ bin }}/taskcluster-proxy
        taskclusterProxyPort: 8080
        livelogExecutable: {{ bin }}/livelog
        wstAudience: taskcluster-net
        wstServerURL: https://websocktunnel.tasks.build
        rootURL: https://taskcluster.net
    - watch_in:
      - service: net.generic.worker

{{ bin }}/generic-worker new-ed25519-keypair --file {{ home }}/keypair:
  cmd.run:
    - creates: {{ home }}/keypair
    - runas: {{ user }}

/Library/LaunchAgents/net.generic.worker.plist:
  file.absent: []

net.generic.worker:
  file.managed:
    - name: /Library/LaunchDaemons/net.generic.worker.plist
    - mode: 600
    - user: root
    - template: jinja
    - source: salt://generic-worker.plist.jinja
    - context:
      bin: {{ bin }}
      etc: {{ etc }}
      home: {{ home }}
      username: {{ user }}
  service.running:
    - enable: True
    - watch:
      - file: /Library/LaunchDaemons/net.generic.worker.plist
