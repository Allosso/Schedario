# Compila la versione definitiva per Windows e la copia in dist\Schedario.exe
# Uso (da PowerShell, nella cartella Note):  .\script\compila-windows.ps1

$ErrorActionPreference = "Stop"
$radice = Split-Path -Parent $PSScriptRoot
Set-Location $radice

# Toolchain Rust (rustup) e MinGW-w64 (dlltool, linker) installati per l'utente.
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:LOCALAPPDATA\Programs\mingw64\bin;$env:Path"

cargo test
if ($LASTEXITCODE -ne 0) { throw "I test non sono passati: compilazione interrotta." }
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Compilazione non riuscita." }

New-Item -ItemType Directory -Force "$radice\dist" | Out-Null
Copy-Item "$radice\target\release\schedario.exe" "$radice\dist\Schedario.exe" -Force
Copy-Item "$radice\LICENSE" "$radice\dist\LICENSE.txt" -Force
Copy-Item "$radice\docs\LICENZE-TERZE-PARTI.md" "$radice\dist\LICENZE-TERZE-PARTI.md" -Force
# Testi completi delle licenze (generati da script\genera-licenze.ps1).
Copy-Item "$radice\assets\licenze\licenze.txt" "$radice\dist\LICENZE.txt" -Force
Write-Host "Pronto: $radice\dist\Schedario.exe"
