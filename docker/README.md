# Building Lemmy Images

Lemmy's images are meant to be **built** on `linux/amd64`,
but they can be **executed** on both `linux/amd64` and `linux/arm64`.

To do so we need to use a _cross toolchain_ whose goal is to build
**from** amd64 **to** arm64.

Namely, we need to link the _lemmy_server_ with `pq` and `openssl`
shared libraries and a few others, and they need to be in `arm64`,
indeed.

The toolchain we use to cross-compile is specifically tailored for
Lemmy's needs, see [the image repository][image-repo].

#### References

- [The Linux Documentation Project on Shared Libraries][tldp-lib]

[tldp-lib]: https://tldp.org/HOWTO/Program-Library-HOWTO/shared-libraries.html
[image-repo]: https://github.com/raskyld/lemmy-cross-toolchains

# Lemmy Docker Deployment Guide

This directory contains the necessary files to deploy Lemmy using Docker Compose.

## Prerequisites

- Docker and Docker Compose installed
- A domain name pointing to your server
- Ports 80 and 443 open on your server

## Deployment Steps

1. **Create required directory structure**

   Create the following directories for volume persistence:
   ```
   mkdir -p volumes/pictrs volumes/postgres volumes/certbot/conf volumes/certbot/www
   ```

2. **Customize configuration files**

   Edit the following files to match your environment:
   
   - `docker-compose.yml`: Update any port mappings if needed
   - `lemmy.hjson`: 
     - Replace `your-domain.com` with your actual domain
     - Set strong passwords for the database and admin user
     - Configure email settings
   - `nginx.conf`: Replace `your-domain.com` with your actual domain

3. **SSL Certificates (Production)**

   For a production environment, obtain SSL certificates from Let's Encrypt:
   
   ```bash
   # Start with HTTP only configuration first
   # Temporarily modify nginx.conf to work without SSL
   
   # Start the services
   docker-compose up -d
   
   # Get certificates (after DNS has propagated)
   docker-compose run --rm certbot certonly --webroot --webroot-path /var/www/certbot -d your-domain.com
   
   # Stop services
   docker-compose down
   
   # Restore original nginx.conf with HTTPS configuration
   
   # Start services again
   docker-compose up -d
   ```

4. **Start Lemmy**

   ```bash
   docker-compose up -d
   ```

5. **Access your Lemmy instance**

   Visit your domain in a web browser and log in with the admin credentials set in `lemmy.hjson`.

## Maintenance

- **Backups**: 
  ```bash
  # Backup database
  docker-compose exec postgres pg_dump -U lemmy lemmy > backup_$(date +%Y%m%d).sql
  ```

- **Updates**:
  ```bash
  # Pull latest images
  docker-compose pull
  
  # Restart services
  docker-compose up -d
  ```

## Troubleshooting

- Check logs: `docker-compose logs`
- Service-specific logs: `docker-compose logs [service-name]`
- Verify configuration files for correctness

## Security Considerations

- Always use strong passwords
- Keep your server and Docker updated
- Consider using a firewall (e.g., UFW)
- Set up automated backups
