# MySQL 

with podman
```bash
podman run --name mysql -e MYSQL_ROOT_PASSWORD=q1w2e3r4 -p 44444:3306 -d mysql
podman exec -it mysql mysql -u root -p
```

run
```bash
magiceye init
# ...
Enter Base Connection URL: mysql://root:q1w2e3r4@127.0.0.1:44444/prod
Enter Target Connection URL: mysql://root:q1w2e3r4@127.0.0.1:44444/dev
# ...

magiceye run
```