# Lemmy Status Check Script
# This script checks the status of your Lemmy deployment

Write-Host "Checking Docker services status..." -ForegroundColor Green
try {
    docker-compose ps
    
    Write-Host "`nChecking container logs (last 10 lines of each service):" -ForegroundColor Green
    
    $services = @("lemmy", "lemmy-ui", "postgres", "pictrs", "nginx")
    
    foreach ($service in $services) {
        Write-Host "`n========== $service logs ==========" -ForegroundColor Cyan
        docker-compose logs --tail=10 $service
    }
    
    Write-Host "`nChecking database connection..." -ForegroundColor Green
    $dbStatus = docker-compose exec -T postgres pg_isready -U lemmy
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Database connection is OK." -ForegroundColor Green
    } else {
        Write-Host "Database connection issue detected." -ForegroundColor Red
    }
    
    Write-Host "`nChecking network connectivity between containers..." -ForegroundColor Green
    docker-compose exec -T lemmy curl -s http://lemmy-ui:1234 > $null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Network connectivity to lemmy-ui is OK." -ForegroundColor Green
    } else {
        Write-Host "Network connectivity issue to lemmy-ui detected." -ForegroundColor Red
    }
    
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
    Write-Host "Make sure Docker is running and the deployment is active." -ForegroundColor Yellow
}

Write-Host "`nIf you encounter issues, check full logs with:" -ForegroundColor Yellow
Write-Host "docker-compose logs" -ForegroundColor Cyan 