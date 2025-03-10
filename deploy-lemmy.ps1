# Master Lemmy Deployment Script
# This script helps deploy Lemmy by delegating to the detailed script in the docker directory

# Display welcome message
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "        Lemmy Deployment Launcher          " -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This script will help you deploy Lemmy using Docker." -ForegroundColor Yellow
Write-Host ""

# Check if Docker is installed
try {
    $dockerVersion = docker --version
    Write-Host "Docker detected: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host "Docker is not installed or not in PATH. Please install Docker Desktop for Windows." -ForegroundColor Red
    Write-Host "Download from: https://www.docker.com/products/docker-desktop/" -ForegroundColor Yellow
    exit 1
}

# Check if docker-compose is installed
try {
    $composeVersion = docker-compose --version
    Write-Host "Docker Compose detected: $composeVersion" -ForegroundColor Green
} catch {
    Write-Host "Docker Compose is not installed or not in PATH." -ForegroundColor Red
    Write-Host "It should be included with Docker Desktop for Windows." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Deployment Options:" -ForegroundColor Cyan
Write-Host "1. Production Deployment (with domain name)" -ForegroundColor White
Write-Host "2. Local Development/Testing" -ForegroundColor White
Write-Host "3. Exit" -ForegroundColor White
Write-Host ""

$option = Read-Host "Select an option (1-3)"

switch ($option) {
    "1" {
        # Production deployment
        $domain = Read-Host "Enter your domain name (e.g., lemmy.example.com)"
        $adminUsername = Read-Host "Enter admin username"
        $adminPassword = Read-Host "Enter admin password" -AsSecureString
        $adminPasswordText = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($adminPassword))
        $adminEmail = Read-Host "Enter admin email"
        $siteName = Read-Host "Enter site name"
        $dbPassword = Read-Host "Enter database password" -AsSecureString
        $dbPasswordText = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($dbPassword))
        
        $configureEmail = Read-Host "Configure email (SMTP) settings? (y/n)"
        $emailParams = ""
        
        if ($configureEmail -eq "y") {
            $smtpServer = Read-Host "Enter SMTP server"
            $smtpPort = Read-Host "Enter SMTP port (default: 587)"
            $smtpUsername = Read-Host "Enter SMTP username"
            $smtpPassword = Read-Host "Enter SMTP password" -AsSecureString
            $smtpPasswordText = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($smtpPassword))
            $smtpFromAddress = Read-Host "Enter 'From' email address"
            
            $emailParams = "-SmtpServer `"$smtpServer`" -SmtpPort `"$smtpPort`" -SmtpUsername `"$smtpUsername`" -SmtpPassword `"$smtpPasswordText`" -SmtpFromAddress `"$smtpFromAddress`""
        }
        
        # Run the deployment script
        Push-Location docker
        $deployCmd = ".\deploy.ps1 -Domain `"$domain`" -AdminUsername `"$adminUsername`" -AdminPassword `"$adminPasswordText`" -AdminEmail `"$adminEmail`" -SiteName `"$siteName`" -DbPassword `"$dbPasswordText`" $emailParams"
        Invoke-Expression $deployCmd
        
        $startServices = Read-Host "Start Lemmy services now? (y/n)"
        if ($startServices -eq "y") {
            docker-compose up -d
            Write-Host "Services started. Check status with .\check-status.ps1" -ForegroundColor Green
        } else {
            Write-Host "To start services later, run 'docker-compose up -d' in the docker directory" -ForegroundColor Yellow
        }
        
        Pop-Location
    }
    "2" {
        # Local development/testing
        Push-Location docker
        
        $deployCmd = ".\deploy.ps1 -Domain `"localhost`" -AdminUsername `"admin`" -AdminPassword `"password`" -AdminEmail `"admin@example.com`" -SiteName `"Local Lemmy`" -DbPassword `"password`""
        Invoke-Expression $deployCmd
        
        $startServices = Read-Host "Start Lemmy services now? (y/n)"
        if ($startServices -eq "y") {
            # Update nginx.conf for local development
            $nginxConfig = Get-Content -Path "nginx.conf" -Raw
            $nginxConfig = $nginxConfig -replace "listen 443 ssl http2;", "listen 80;"
            $nginxConfig = $nginxConfig -replace "ssl_certificate.*?;", ""
            $nginxConfig = $nginxConfig -replace "ssl_certificate_key.*?;", ""
            $nginxConfig = $nginxConfig -replace "return 301 https://\$host\$request_uri;", "proxy_pass http://lemmy-ui:1234;"
            $nginxConfig | Set-Content -Path "nginx.conf" -Force
            
            # Update docker-compose.yml
            $dockerComposeConfig = Get-Content -Path "docker-compose.yml" -Raw
            $dockerComposeConfig = $dockerComposeConfig -replace "LEMMY_HTTPS=true", "LEMMY_HTTPS=false"
            $dockerComposeConfig | Set-Content -Path "docker-compose.yml" -Force
            
            # Start services
            docker-compose up -d
            Write-Host "Services started. Your local Lemmy instance should be available at http://localhost" -ForegroundColor Green
            Write-Host "Check status with .\check-status.ps1" -ForegroundColor Green
        } else {
            Write-Host "To start services later, run 'docker-compose up -d' in the docker directory" -ForegroundColor Yellow
        }
        
        Pop-Location
    }
    "3" {
        Write-Host "Exiting..." -ForegroundColor Yellow
        exit 0
    }
    default {
        Write-Host "Invalid option. Exiting..." -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "For more information, see DEPLOYMENT.md" -ForegroundColor Yellow 