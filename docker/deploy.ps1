# Lemmy Deployment PowerShell Script
# This script automates the deployment of Lemmy on Windows

# Parameters
param(
    [Parameter(Mandatory=$true)]
    [string]$Domain,
    
    [Parameter(Mandatory=$true)]
    [string]$AdminUsername,
    
    [Parameter(Mandatory=$true)]
    [string]$AdminPassword,
    
    [Parameter(Mandatory=$true)]
    [string]$AdminEmail,
    
    [Parameter(Mandatory=$true)]
    [string]$SiteName,
    
    [Parameter(Mandatory=$true)]
    [string]$DbPassword,
    
    [Parameter(Mandatory=$false)]
    [string]$SmtpServer = "",
    
    [Parameter(Mandatory=$false)]
    [string]$SmtpPort = "587",
    
    [Parameter(Mandatory=$false)]
    [string]$SmtpUsername = "",
    
    [Parameter(Mandatory=$false)]
    [string]$SmtpPassword = "",
    
    [Parameter(Mandatory=$false)]
    [string]$SmtpFromAddress = ""
)

# Create directory structure
Write-Host "Creating directory structure..." -ForegroundColor Green
New-Item -Path "volumes\pictrs" -ItemType Directory -Force | Out-Null
New-Item -Path "volumes\postgres" -ItemType Directory -Force | Out-Null
New-Item -Path "volumes\certbot\conf" -ItemType Directory -Force | Out-Null
New-Item -Path "volumes\certbot\www" -ItemType Directory -Force | Out-Null

# Update configuration files with provided parameters
Write-Host "Updating configuration files..." -ForegroundColor Green

# Update lemmy.hjson
$lemmyConfig = Get-Content -Path "lemmy.hjson" -Raw
$lemmyConfig = $lemmyConfig -replace "your-domain.com", $Domain
$lemmyConfig = $lemmyConfig -replace '"admin"', "`"$AdminUsername`""
$lemmyConfig = $lemmyConfig -replace '"change_me"', "`"$AdminPassword`""
$lemmyConfig = $lemmyConfig -replace '"admin@example.com"', "`"$AdminEmail`""
$lemmyConfig = $lemmyConfig -replace '"My Lemmy Instance"', "`"$SiteName`""
$lemmyConfig = $lemmyConfig -replace '"password"', "`"$DbPassword`""

# Update email settings if provided
if ($SmtpServer -ne "") {
    $lemmyConfig = $lemmyConfig -replace '"smtp.example.com"', "`"$SmtpServer`""
    $lemmyConfig = $lemmyConfig -replace '"587"', "`"$SmtpPort`""
    $lemmyConfig = $lemmyConfig -replace '"noreply@example.com"', "`"$SmtpFromAddress`""
    $lemmyConfig = $lemmyConfig -replace 'smtp_login: "noreply@example.com"', "smtp_login: `"$SmtpUsername`""
    $lemmyConfig = $lemmyConfig -replace 'smtp_password: "password"', "smtp_password: `"$SmtpPassword`""
}

$lemmyConfig | Set-Content -Path "lemmy.hjson" -Force

# Update docker-compose.yml
$dockerComposeConfig = Get-Content -Path "docker-compose.yml" -Raw
$dockerComposeConfig = $dockerComposeConfig -replace "your-domain.com", $Domain
$dockerComposeConfig = $dockerComposeConfig -replace "POSTGRES_PASSWORD=password", "POSTGRES_PASSWORD=$DbPassword"
$dockerComposeConfig | Set-Content -Path "docker-compose.yml" -Force

# Update nginx.conf
$nginxConfig = Get-Content -Path "nginx.conf" -Raw
$nginxConfig = $nginxConfig -replace "your-domain.com", $Domain
$nginxConfig | Set-Content -Path "nginx.conf" -Force

# Check if Docker is installed
Write-Host "Checking Docker installation..." -ForegroundColor Green
try {
    docker --version | Out-Null
    Write-Host "Docker is installed." -ForegroundColor Green
} catch {
    Write-Host "Docker is not installed or not in PATH. Please install Docker Desktop for Windows." -ForegroundColor Red
    exit 1
}

Write-Host "Configuration complete!" -ForegroundColor Green
Write-Host "--------------------------------" -ForegroundColor Yellow
Write-Host "To deploy Lemmy, run:" -ForegroundColor Yellow
Write-Host "docker-compose up -d" -ForegroundColor Cyan
Write-Host "--------------------------------" -ForegroundColor Yellow
Write-Host "For production deployment, follow the SSL certificate instructions in README.md" -ForegroundColor Yellow 