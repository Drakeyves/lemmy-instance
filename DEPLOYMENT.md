# Lemmy Deployment Guide

This guide provides instructions for deploying Lemmy on Windows using Docker.

## Prerequisites

- Windows 10/11 with PowerShell
- [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop/) installed and running
- A domain name pointing to your server (for production deployments)
- Open ports 80 and 443 on your server/router (for production deployments)

## Deployment Options

### Option 1: Quick Start with PowerShell Script (Recommended)

1. Navigate to the `docker` directory:
   ```powershell
   cd docker
   ```

2. Run the deployment script with your settings:
   ```powershell
   .\deploy.ps1 -Domain "your-domain.com" -AdminUsername "admin" -AdminPassword "secure_password" -AdminEmail "admin@example.com" -SiteName "My Lemmy Instance" -DbPassword "secure_db_password"
   ```

   For email configuration, add these parameters:
   ```powershell
   -SmtpServer "smtp.example.com" -SmtpPort "587" -SmtpUsername "username" -SmtpPassword "password" -SmtpFromAddress "noreply@example.com"
   ```

3. Start the services:
   ```powershell
   docker-compose up -d
   ```

4. Check status:
   ```powershell
   .\check-status.ps1
   ```

### Option 2: Manual Setup

1. Create required directories:
   ```powershell
   mkdir -p docker/volumes/pictrs docker/volumes/postgres docker/volumes/certbot/conf docker/volumes/certbot/www
   ```

2. Edit configuration files in the `docker` directory:
   - `docker-compose.yml`: Update domain and other settings
   - `lemmy.hjson`: Configure your instance settings
   - `nginx.conf`: Update domain name

3. Start the services:
   ```powershell
   cd docker
   docker-compose up -d
   ```

## Production Deployment Considerations

For a production deployment, you'll need SSL certificates. After setting up your DNS to point to your server:

1. Temporarily modify `nginx.conf` to work without SSL (HTTP only)
2. Start services: `docker-compose up -d`
3. Get certificates: 
   ```
   docker-compose run --rm certbot certonly --webroot --webroot-path /var/www/certbot -d your-domain.com
   ```
4. Stop services: `docker-compose down`
5. Restore the original `nginx.conf` with HTTPS configuration
6. Start services again: `docker-compose up -d`

## Testing Your Deployment

1. Local development: Access http://localhost
2. Production: Access https://your-domain.com

## Maintenance

### Backups

```powershell
# Backup the database
cd docker
docker-compose exec postgres pg_dump -U lemmy lemmy > ../backups/lemmy_backup_$(Get-Date -Format "yyyyMMdd").sql

# Backup configuration
Copy-Item docker-compose.yml, lemmy.hjson, nginx.conf ../backups/
```

### Updates

```powershell
cd docker
docker-compose pull
docker-compose up -d
```

## Troubleshooting

If you encounter issues:

1. Check service status: 
   ```
   .\check-status.ps1
   ```
   
2. View detailed logs:
   ```
   docker-compose logs
   ```
   
3. Check specific service logs:
   ```
   docker-compose logs lemmy
   ```

4. Verify database connection:
   ```
   docker-compose exec postgres pg_isready -U lemmy
   ```

5. Check if services can communicate:
   ```
   docker-compose exec lemmy curl -s http://lemmy-ui:1234
   ```

## Security Best Practices

1. Use strong passwords for admin and database accounts
2. Keep Docker and your host system updated
3. Set up automated backups
4. Use a firewall to restrict access to only necessary ports
5. Regularly monitor logs for suspicious activity

## References

- [Lemmy Documentation](https://join-lemmy.org/docs/index.html)
- [Docker Documentation](https://docs.docker.com/)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/) 