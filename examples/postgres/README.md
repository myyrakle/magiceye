# PostgreSQL

with podman
```bash
podman run --name psql -e POSTGRES_PASSWORD=q1w2e3r4 -d postgres
podman exec -it psql psql -U postgres -d postgres
```