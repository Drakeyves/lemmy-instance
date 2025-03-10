# Troubleshooting and Fixing the Monitoring Proxy

This document details the issue we encountered with the monitoring proxy container and how we resolved it.

## The Problem

After setting up our Lemmy instance with a monitoring proxy to capture backend traffic, we noticed that while most containers were starting successfully, the `docker-proxy-1` container was stuck in a restart loop:

```
docker-pictrs-1      Running
docker-postgres-1    Running
docker-lemmy-1       Running
docker-lemmy-ui-1    Running
docker-proxy-1       Restarting
```

The container was unable to start properly and kept restarting, preventing us from monitoring backend traffic as intended.

## Diagnosis

We checked the logs of the failing container to identify the issue:

```bash
docker logs docker-proxy-1
```

The logs revealed the following error:

```
nginx: [emerg] unknown directive "lua_package_path" in /etc/nginx/nginx.conf:19
```

This error indicated that the NGINX configuration was using Lua scripting features, but the container didn't have Lua support installed. The specific error was pointing to our attempt to use `lua_package_path` directive, which is only available in OpenResty (an NGINX distribution with Lua support), not in the standard NGINX image.

## Root Cause

The issue stemmed from a mismatch between our configuration and the Docker image:

1. Our NGINX configuration was using advanced Lua scripting for capturing response bodies and detailed logging
2. We were trying to use these Lua features with the standard `nginx:alpine` image
3. The standard NGINX image doesn't include Lua modules, so it couldn't interpret the Lua directives

Additionally, while our `docker-compose.yml` was specifying `openresty/openresty:alpine` as the image, there was a problem with how the image was being pulled or used.

## Solution

We implemented a two-part solution:

1. **Simplified the NGINX configuration**:
   - Removed all Lua-dependent code
   - Used standard NGINX logging without response body capture
   - Maintained the proxy functionality to forward traffic to the Lemmy backend

2. **Updated the Docker image**:
   - Explicitly switched to the standard `nginx:alpine` image
   - Ensured our configuration was compatible with this image

### Configuration Changes

#### Original Configuration (with Lua)
```nginx
# Enhanced logging format with Lua
log_format detailed_log '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent" '
                        '$request_time '
                        'request_body="$request_body" '
                        'response="$upstream_response_body"';

# Lua package path
lua_package_path "/etc/nginx/lua/?.lua;;";

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
```

#### New Configuration (Standard NGINX)
```nginx
# Basic logging format
log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                '$status $body_bytes_sent "$http_referer" '
                '"$http_user_agent" "$http_x_forwarded_for"';

# Access log location
access_log /var/log/nginx/access.log main;
error_log /var/log/nginx/error.log;

# MIME types
include /etc/nginx/mime.types;
default_type application/octet-stream;
```

## Results

After implementing these changes and restarting the containers:

```bash
docker-compose down
docker-compose up -d
```

All containers successfully started, including the monitoring proxy:

```
docker-pictrs-1      Running
docker-postgres-1    Running
docker-lemmy-1       Running
docker-lemmy-ui-1    Running
docker-proxy-1       Running
```

## Impact on Monitoring Capabilities

The changes did reduce our monitoring capabilities slightly:

**Original Plan (with Lua):**
- Would have captured full request and response bodies
- Could have inspected the content of requests and responses
- More detailed for debugging API calls

**Current Implementation (Standard NGINX):**
- Captures all request URLs, methods, status codes
- Logs user agents and referrers
- Sufficient for monitoring which endpoints are being called
- Doesn't capture the actual request/response payloads

## Future Improvements

If detailed payload monitoring is needed in the future, we could:

1. **Use OpenResty properly**: Ensure the OpenResty image is correctly configured and used
2. **Add a dedicated API monitoring tool**: Implement a tool like Prometheus or Grafana for more detailed monitoring
3. **Create a custom monitoring service**: Build a custom service that sits between clients and Lemmy to log full request/response details

## Lessons Learned

1. **Match configuration to container capabilities**: Ensure your configuration uses features supported by your chosen container image
2. **Start with simpler solutions**: Begin with basic configurations that are likely to work, then add complexity
3. **Check container logs immediately**: Container logs quickly reveal configuration problems
4. **Understand the difference between NGINX variants**: Regular NGINX and OpenResty have different capabilities 