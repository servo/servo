{% set bin = "/usr/local/bin" %}
{% set user = "worker" %}
{% set home = "/Users/" + user %}

{{ bin }}/generic-worker:
  file.managed:
    - name:
    - source: https://github.com/taskcluster/generic-worker/releases/download/v11.0.1/generic-worker-darwin-amd64
    - source_hash: sha256=059331865670d3722a710f0b6f4dae97d347811cc347d1810c6dfc1b413c4b48
    - mode: 755
    - makedirs: True

{{ bin }}/livelog:
  file.managed:
    - source: https://github.com/taskcluster/livelog/releases/download/v1.1.0/livelog-darwin-amd64
    - source_hash: sha256=be5d4b998b208afd802ac6ce6c4d4bbf0fb3816bb039a300626abbc999dfe163
    - mode: 755
    - makedirs: True

{{ user }}:
  user.present:
    - home: {{ home }}

# `user.present`’s `createhome` is apparently not supported on macOS
{{ home }}:
  file.directory:
    - user: {{ user }}

{{ home }}/config.json:
  file.serialize:
    - makedirs: True
    - user: {{ user }}
    - mode: 600
    - show_changes: False
    - formatter: json
    - dataset:
        provisionerId: proj-servo
        workerType: macos
        workerGroup: servo-macos
        workerId: mac1
        tasksDir: {{ home }}/tasks
        publicIP: {{ salt.network.ip_addrs()[0] }}
        signingKeyLocation: {{ home }}/key
        clientId: {{ pillar["client_id"] }}
        accessToken: {{ pillar["access_token"] }}
        livelogSecret: {{ pillar["livelog_secret"] }}

{{ bin }}/generic-worker new-openpgp-keypair --file {{ home }}/key:
  cmd.run:
    - creates: {{ home }}/key
    - runas: worker

/Library/LaunchAgents/net.generic.worker.plist:
  file.managed:
    - mode: 644
    - template: jinja
    - contents: >-
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
          <key>Label</key>
          <string>net.generic.worker</string>

          <key>ProgramArguments</key>
          <array>
            <string>{{ bin }}/generic-worker</string>
            <string>run</string>
            <string>--config</string>
            <string>config.json</string>
          </array>

          <key>KeepAlive</key>
          <true/>

          <key>WorkingDirectory</key>
          <string>{{ home }}</string>

          <key>UserName</key>
          <string>{{ user }}</string>

          <key>StandardOutPath</key>
          <string>stdout.log</string>

          <key>StandardErrorPath</key>
          <string>stderr.log</string>
        </dict>
        </plist>

net.generic.worker:
  service.running:
    - enable: True
