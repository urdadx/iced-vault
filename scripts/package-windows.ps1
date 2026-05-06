$ErrorActionPreference = 'Stop'

New-Item -ItemType Directory -Force -Path dist | Out-Null
Remove-Item -Force -ErrorAction SilentlyContinue dist\*.exe

cargo build --release

$target = 'dist\iced-vault-windows-x86_64.exe'
Copy-Item 'target\release\iced-vault.exe' $target -Force

Write-Host "Created $target"
