# Lemmy Database Connection Issue Fix

## Problem Description

**Date/Time: March 10, 2025 01:25:16**

The Lemmy application was failing to start with the following error:

```
Error connecting to postgres://lemmy:password@localhost:5432/lemmy: connection to server at "localhost" (::1), port 5432 failed: Connection refused
```

**Root cause:** 
In the Docker environment, Lemmy was attempting to connect to PostgreSQL using `localhost` as the host, but in containerized environments, services need to reference each other by their service name (`postgres`) instead of `localhost`.

## Troubleshooting Timeline

### 01:29:06 - Initial Assessment
- Stopped all running Docker containers to get a clean slate
- Observed the container logs showing connection errors to `localhost` despite the configuration file using the correct `postgres` hostname

### 01:29:20 - First Attempt
- Restarted all containers to see if it was a transient issue
- Inspected logs showing Lemmy was still trying to connect to `localhost`:
```
Error connecting to postgres://lemmy:password@localhost:5432/lemmy: connection to server at "localhost" (::1), port 5432 failed: Connection refused
```

### 01:29:35 - Confirming Configuration
- Examined the `lemmy.hjson` file which correctly had:
```
database: {
  connection: "postgres://lemmy:password@postgres:5432/lemmy"
  pool_size: 10
}
```
- Noted that despite correct configuration, Lemmy was ignoring it and using `localhost`

### 01:29:44 - Further Investigation
- Attempted to inspect container configuration but found it in a restart loop
- The container kept crashing due to the database connection failure

### 01:30:11 - Solution Implementation
- Modified `docker-compose.yml` to add an environment variable that would override the configuration file:
```yaml
environment:
  - RUST_LOG=warn
  - LEMMY_DATABASE_URL=postgres://lemmy:password@postgres:5432/lemmy
```

### 01:30:18 - Applying the Fix
- Stopped all containers again
- Started containers with the updated configuration

### 01:30:45 - Verification
- Checked Lemmy service logs
- Confirmed successful startup:
```
Lemmy v0.19.3
Federation enabled, host is localhost
Starting HTTP server at 0.0.0.0:8536
```

## Solution

The fix was to add an explicit environment variable `LEMMY_DATABASE_URL` to the Lemmy service in `docker-compose.yml`:

```yaml
lemmy:
  image: dessalines/lemmy:0.19.3
  hostname: lemmy
  restart: always
  ports:
    - "8536:8536"
  environment:
    - RUST_LOG=warn
    - LEMMY_DATABASE_URL=postgres://lemmy:password@postgres:5432/lemmy
  volumes:
    - ./lemmy.hjson:/config/config.hjson
  depends_on:
    postgres:
      condition: service_healthy
    pictrs:
      condition: service_started
  logging: *default-logging
```

This environment variable overrides any settings in the configuration file, ensuring Lemmy connects to the database using the service name `postgres` rather than `localhost`.

## Lessons Learned & Best Practices

1. **Container Networking:** In Docker environments, services should communicate using service names as defined in `docker-compose.yml`, not `localhost`.

2. **Configuration Priority:** Environment variables typically take precedence over configuration files in containerized applications.

3. **Troubleshooting Approach:** When investigating connection issues between containers:
   - Check if the target service is running (PostgreSQL was running fine)
   - Verify connection details are correct for the environment (using container name vs localhost)
   - Examine logs for specific errors ("Connection refused" indicated wrong hostname)

4. **Environment Variables as Solution:** When configuration files don't seem to take effect, environment variables can provide a more direct way to configure container behavior.

## How to Access the Application

The Lemmy instance is now accessible at:
- http://localhost (via the nginx proxy)
- http://localhost:8536 (direct to Lemmy backend)
- http://localhost:1234 (direct to Lemmy UI)

Login credentials:
- Username: admin
- Password: password 