# PostgreSQL

with podman
```bash
podman run --name psql -e POSTGRES_PASSWORD=q1w2e3r4 -p 44444:5432 postgres
podman exec -it psql psql -U postgres -d postgres
```

run
```bash
magiceye init
# ...
Enter Base Connection URL: postgres://postgres:q1w2e3r4@localhost:44444/prod
Enter Target Connection URL: postgres://postgres:q1w2e3r4@localhost:44444/dev
# ...

magiceye run
```