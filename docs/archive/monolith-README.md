Monolith (v0.1)

Run:
  python3 monolith/daemon.py --host 127.0.0.1 --port 8085

API:
  POST { "action":"lookup", "hash":"<sha256>", "unit":"module" }
  POST { "action":"store", "hash":"<sha256>", "unit":"module", "artifact": {...} }
  POST { "action":"diagnostics", "project":"default" }

This daemon matches docs/MONOLITH_PROTOCOL.md.
