# Lemmy Backend Setup and Monitoring Guide

This document provides a comprehensive guide on how the Lemmy backend is configured and how to access all backend data without modifying the core code.

## Table of Contents

1. [Backend Architecture](#backend-architecture)
2. [Accessing Backend as Admin](#accessing-backend-as-admin)
3. [Logging Configuration](#logging-configuration)
4. [Webhook Notifications](#webhook-notifications)
5. [Traffic Monitoring Proxy](#traffic-monitoring-proxy)
6. [Direct Database Access](#direct-database-access)
7. [Troubleshooting](#troubleshooting)

## Backend Architecture

Lemmy's backend consists of:

- **Programming Language**: Rust
- **Web Framework**: Actix
- **Database ORM**: Diesel
- **Database**: PostgreSQL
- **Frontend**: Inferno.js with TypeScript
- **API**: REST API endpoints
- **Federation**: ActivityPub protocol

The deployment uses Docker Compose with these key services:
- PostgreSQL database
- Lemmy backend
- Lemmy UI
- Monitoring proxy (OpenResty with NGINX)

## Accessing Backend as Admin

The simplest way to access the backend is through the admin account:

1. **Access URL**: http://localhost:1234
2. **Login credentials**:
   - Username: `admin`
   - Password: `password1234`

After logging in, you can:
- View all site content
- Moderate posts and comments
- Ban users
- Manage communities
- Configure site settings

## Logging Configuration

We've set up detailed logging to capture all backend activity:

### Configuration
```hjson
logging: {
  level: debug
  log_file: "/app/lemmy.log"
}
```

### Log Files Location
Logs are stored in the `logs` directory:
- Lemmy application logs: `logs/lemmy.log`
- Volume mount in docker-compose: `./logs:/app/logs`

### Log Contents
The logs capture:
- HTTP requests
- Database queries
- Authentication events
- Federation activity (when enabled)
- Error messages

### How to View Logs
```bash
# Tail the logs in real-time
tail -f logs/lemmy.log

# Search logs for specific activity
grep "POST" logs/lemmy.log
```

## Webhook Notifications

We've added webhook configuration to receive notifications for all backend activity:

### Configuration
```hjson
webhook: {
  url: "https://your-webhook-url.com/api/receive"
  username: "lemmy-notifier"
  password: "your-secure-password"
  send_all_activities: true
}
```

### Setting Up a Webhook Receiver
You need to:
1. Replace `https://your-webhook-url.com/api/receive` with your actual webhook endpoint URL
2. Change the username and password for webhook authentication
3. Ensure your webhook receiver can handle POST requests with JSON payloads

### Webhook Payload Example
```json
{
  "activity_type": "CreatePost",
  "community_id": 2,
  "user_id": 1,
  "content": {
    "post_id": 5,
    "title": "Post Title",
    "content": "Post content..."
  },
  "timestamp": "2023-07-01T12:34:56Z"
}
```

## Traffic Monitoring Proxy

We've set up an NGINX-based monitoring proxy to capture all HTTP traffic:

### Configuration
The proxy is configured in `docker/nginx.conf` with:
- Standard NGINX access and error logging
- All requests forwarded to the Lemmy backend
- Access on port 8080

### How It Works
1. All requests to the proxy (localhost:8080) are forwarded to the Lemmy backend
2. The proxy logs each request's:
   - Client IP
   - Request details (path, method)
   - Response status
   - User agent
   - Referrer

### Log Format
```
$remote_addr - $remote_user [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for"
```

### Viewing Proxy Logs
```bash
# View the access logs
cat logs/nginx/access.log

# Monitor logs in real-time
tail -f logs/nginx/access.log

# Filter for specific API endpoints
grep "/api/v3/post" logs/nginx/access.log
```

## Direct Database Access

You can directly access the PostgreSQL database for advanced queries:

### Connection Details
- **Host**: localhost
- **Port**: 5432
- **Database**: lemmy
- **Username**: lemmy
- **Password**: password

### Connecting to the Database
```bash
# Connect using psql
psql -h localhost -p 5432 -U lemmy -d lemmy

# Connect using a GUI tool like pgAdmin, DBeaver, etc.
# Use the connection details above
```

### Important Database Tables
- `user_`: User accounts
- `post`: Posts/submissions
- `comment`: Comments on posts
- `community`: Communities/subreddits
- `site`: Site configuration
- `local_user`: Local user settings
- `moderator`: Community moderators
- `comment_like`: Comment votes
- `post_like`: Post votes

### Sample Queries
```sql
-- Get all posts
SELECT * FROM post;

-- Get all users
SELECT * FROM user_;

-- Get all communities
SELECT * FROM community;

-- Get recent activity
SELECT * FROM post ORDER BY published DESC LIMIT 10;
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Logs not appearing
Check the permissions of the logs directory:
```bash
chmod -R 777 logs/
```

#### 2. Webhook not receiving data
- Verify the webhook URL is accessible from the Docker container
- Check webhook credentials
- Ensure the webhook endpoint can accept POST requests

#### 3. Monitoring proxy issues
If the proxy isn't capturing traffic:
```bash
# Check if the container is running
docker-compose ps monitoring-proxy

# View proxy logs
docker-compose logs monitoring-proxy
```

#### 4. Database connection issues
```bash
# Check if PostgreSQL is running
docker-compose ps postgres

# Check PostgreSQL logs
docker-compose logs postgres
```

#### 5. Restarting services
```bash
# Restart a specific service
docker-compose restart lemmy

# Restart all services
docker-compose down
docker-compose up -d
```

## Docker Compose Configuration

The complete `docker-compose.yml` configuration:

```yaml
version: '3'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_PASSWORD=password
      - POSTGRES_USER=lemmy
      - POSTGRES_DB=lemmy
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: always
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U lemmy"]
      interval: 5s
      timeout: 5s
      retries: 5

  lemmy:
    image: dessalines/lemmy:0.19.3
    environment:
      - RUST_LOG=debug
      - LEMMY_DATABASE_URL=postgres://lemmy:password@postgres:5432/lemmy
    ports:
      - "8536:8536"
    volumes:
      - ./config:/config
      - ./logs:/app/logs
    depends_on:
      postgres:
        condition: service_healthy
    restart: always
    healthcheck:
      test: ["CMD-SHELL", "wget -qO- http://localhost:8536/api/v3/site || exit 1"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  lemmy-ui:
    image: dessalines/lemmy-ui:0.19.3
    environment:
      - LEMMY_INTERNAL_HOST=lemmy:8536
      - LEMMY_EXTERNAL_HOST=your-ip-or-domain:8536
      - LEMMY_UI_DEBUG=true
      - LEMMY_HTTPS=false
    ports:
      - "1234:1234"
    depends_on:
      - lemmy
    restart: always

  monitoring-proxy:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./logs/nginx:/var/log/nginx
    depends_on:
      - lemmy
    restart: always

volumes:
  postgres_data:
```

## NGINX Monitoring Configuration

The complete NGINX configuration for traffic monitoring:

```nginx
worker_processes 1;
events {
    worker_connections 1024;
}

http {
    # Enhanced logging format that includes request and response data
    log_format detailed_log '$remote_addr - $remote_user [$time_local] '
                            '"$request" $status $body_bytes_sent '
                            '"$http_referer" "$http_user_agent" '
                            '$request_time '
                            'request_body="$request_body" '
                            'response="$upstream_response_body"';
    
    # Set the log file location
    access_log /var/log/nginx/detailed_access.log detailed_log;

    # Buffer for reading the response body
    lua_package_path "/etc/nginx/lua/?.lua;;";
    
    server {
        listen 80;
        server_name localhost;

        # Capture request body
        lua_need_request_body on;
        
        # Add response body filter
        body_filter_by_lua_block {
            local resp_body = string.sub(ngx.arg[1], 1, 1000)
            ngx.ctx.resp_body = (ngx.ctx.resp_body or "") .. resp_body
            if ngx.arg[2] then
                ngx.var.upstream_response_body = ngx.ctx.resp_body
            end
        }

        location / {
            proxy_pass http://lemmy:8536;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
``` 