# Lemmy Docker Deployment Guide

This guide will walk you through setting up Lemmy (an open-source link aggregator and forum) using Docker Compose. Follow these instructions to get your local instance up and running quickly.

## Prerequisites

- **Docker Engine**: Version 19.03.0 or newer
- **Docker Compose**: Version 1.27.0 or newer

### Installing Prerequisites

**Windows**:
- Install [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/)

**macOS**:
- Install [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/)

**Linux**:
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.15.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

## Directory Structure

Ensure your directory structure looks like this:
```
.
├── docker/
│   ├── docker-compose.yml
│   ├── lemmy.hjson
│   ├── nginx.conf
│   └── volumes/
│       ├── pictrs/
│       └── postgres/
```

If the `volumes` directory doesn't exist, create it:
```bash
mkdir -p docker/volumes/pictrs docker/volumes/postgres
```

## Configuration Files

The main configuration files are:

1. **docker-compose.yml**: Defines all services
2. **lemmy.hjson**: Lemmy application configuration
3. **nginx.conf**: Web server configuration

### Important Configuration Notes

Make sure the database connection in `docker-compose.yml` uses the service name:
```yaml
environment:
  - LEMMY_DATABASE_URL=postgres://lemmy:password@postgres:5432/lemmy
```

## Deployment Steps

1. **Navigate to the docker directory**:
   ```bash
   cd docker
   ```

2. **Start the Docker containers**:
   ```bash
   docker-compose up -d
   ```

3. **Check if all services are running**:
   ```bash
   docker-compose ps
   ```

   You should see all services (lemmy, lemmy-ui, postgres, pictrs, proxy) in the "Up" state.

4. **View logs to verify successful startup**:
   ```bash
   docker-compose logs lemmy
   ```

   Look for: "Starting HTTP server at 0.0.0.0:8536"

## Accessing Lemmy

Once everything is running:

- **Main URL**: http://localhost
- **Direct Backend**: http://localhost:8536
- **Direct UI**: http://localhost:1234

### Default Login Credentials

- **Username**: admin
- **Password**: password

## Troubleshooting

### Common Issues

1. **Database connection errors**:
   - Ensure PostgreSQL container is running: `docker-compose ps postgres`
   - Check the database connection URL in both `lemmy.hjson` and `docker-compose.yml`
   - Make sure the connection URL uses `postgres` as hostname, not `localhost`

2. **Service won't start**:
   - Check for port conflicts: `netstat -tuln | grep -E '80|8536|1234|8080'`
   - Examine logs: `docker-compose logs <service-name>`

3. **Container restarts repeatedly**:
   - Look for errors: `docker-compose logs --tail=50 <service-name>`
   - Check volume mounts and permissions

### Fixing Issues

If you encounter problems:

1. **Stop all services**:
   ```bash
   docker-compose down
   ```

2. **Remove volumes (if database issues persist)**:
   ```bash
   docker-compose down -v
   ```
   Note: This will delete all data!

3. **Restart with logging**:
   ```bash
   docker-compose up
   ```
   (Without -d to see logs in real-time)

## Managing Your Lemmy Instance

### Stopping Lemmy

```bash
docker-compose down
```

### Updating Lemmy

```bash
docker-compose pull
docker-compose down
docker-compose up -d
```

### Backing Up Data

```bash
# Backup PostgreSQL data
docker-compose exec postgres pg_dump -U lemmy lemmy > lemmy_backup.sql
```

## Additional Resources

- [Official Lemmy Documentation](https://join-lemmy.org/docs/)
- [Lemmy GitHub Repository](https://github.com/LemmyNet/lemmy)
- [Docker Compose Documentation](https://docs.docker.com/compose/)

## Security Notice

This setup is configured for local development/testing. For production:
- Change all default passwords
- Configure HTTPS/SSL
- Set up proper data backup
- Configure email for user registration/recovery 