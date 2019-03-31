
**store saltstack pillar info to consul db**

## usage

* clone repo

    `git clone https://github.com/vinian/store_pillar_to_consul.git`

* dump saltstack pillar db as json (currently only support json)

    `salt-call pillar.items --out=json >pillar.json`

* run the program

    `cd store_pillar_to_consul; CONSUL_TOKEN=$consul_token CONSUL_KV_PREFIX='salt-shared' CONSUL_HOST=$consul_http_host cargo run /path/to/saltstack-pillar.json`

CONSUL_TOKEN is the token consul use to access it db
CONSUL_KV_PREFIX is the kv prefix use to store the key
CONSUL_HOST is the consul http api host


