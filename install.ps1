#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$repo = 'hodstack/hodstack'
$dir = if ($env:HOD_INSTALL_DIR) { $env:HOD_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'hod\bin' }
$url = if ($env:HOD_TAG) {
    "https://github.com/$repo/releases/download/$env:HOD_TAG"
} else {
    "https://github.com/$repo/releases/latest/download"
}

function Get-Target {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        'AMD64' { 'x86_64-pc-windows-msvc' }
        default { throw "no build for $env:PROCESSOR_ARCHITECTURE" }
    }
}

$target = Get-Target
$file = "hod-$target.tar.gz"
$work = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.Guid]::NewGuid()))

try {
    Write-Host "Downloading hod for $target"
    Invoke-WebRequest -Uri "$url/$file" -OutFile (Join-Path $work $file) -UseBasicParsing
    Invoke-WebRequest -Uri "$url/checksums.txt" -OutFile (Join-Path $work 'checksums.txt') -UseBasicParsing

    $line = Select-String -Path (Join-Path $work 'checksums.txt') -Pattern " $file$"
    if (-not $line) { throw "checksums.txt names no $file" }

    $wanted = $line.Line.Split(' ')[0]
    $found = (Get-FileHash -Path (Join-Path $work $file) -Algorithm SHA256).Hash.ToLower()
    if ($wanted -ne $found) { throw "the checksum of $file does not agree with checksums.txt" }

    tar -xzf (Join-Path $work $file) -C $work hod.exe
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
    Move-Item -Path (Join-Path $work 'hod.exe') -Destination (Join-Path $dir 'hod.exe') -Force

    Write-Host "Installed $(& (Join-Path $dir 'hod.exe') --version)"
    Write-Host "          $(Join-Path $dir 'hod.exe')"

    $path = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($path -notlike "*$dir*") {
        [Environment]::SetEnvironmentVariable('Path', "$dir;$path", 'User')
        Write-Host ''
        Write-Host "Added $dir to your PATH. Open a new terminal."
    }
}
finally {
    Remove-Item -Path $work -Recurse -Force -ErrorAction SilentlyContinue
}
