# Test Database Connection Script for Lemmy

Write-Host "Testing database connection from Lemmy container..." -ForegroundColor Green

# Check if containers are running
$containerStatus = docker-compose ps lemmy postgres
Write-Host "Container Status:" -ForegroundColor Cyan
Write-Host $containerStatus

# Test PostgreSQL directly
Write-Host "`nTesting PostgreSQL readiness..." -ForegroundColor Cyan
docker-compose exec -T postgres pg_isready -U lemmy

# Get PostgreSQL container IP
$pgIp = docker-compose exec -T postgres hostname -i
Write-Host "`nPostgreSQL container IP: $pgIp" -ForegroundColor Cyan

# Test network connectivity from Lemmy to PostgreSQL
Write-Host "`nTesting network connectivity from Lemmy to PostgreSQL..." -ForegroundColor Cyan
docker-compose exec -T lemmy sh -c "ping -c 4 postgres"

# Test PostgreSQL connection from Lemmy container
Write-Host "`nTesting PostgreSQL connection from Lemmy container..." -ForegroundColor Cyan
docker-compose exec -T lemmy sh -c "apt-get update -qq && apt-get install -y postgresql-client"
docker-compose exec -T lemmy sh -c "PGPASSWORD=password psql -h postgres -U lemmy -d lemmy -c 'SELECT 1'"

# Check Lemmy configuration
Write-Host "`nChecking Lemmy configuration..." -ForegroundColor Cyan
docker-compose exec -T lemmy sh -c "cat /config/config.hjson | grep -A 3 database"

# Restart Lemmy service with debug logs
Write-Host "`nRestarting Lemmy service with debug logs..." -ForegroundColor Cyan
docker-compose stop lemmy
docker-compose rm -f lemmy
docker-compose up -d lemmy

# Display Lemmy logs
Write-Host "`nLemmy logs (wait a few seconds for startup):" -ForegroundColor Cyan
Start-Sleep -Seconds 3
docker-compose logs --tail=30 lemmy

Write-Host "`nTroubleshooting complete. If issues persist, check logs with 'docker-compose logs lemmy'" -ForegroundColor Yellow 