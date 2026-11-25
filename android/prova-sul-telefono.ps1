# Installa e avvia Schedario sul telefono collegato via USB (con il "debug
# USB" attivo nelle Opzioni sviluppatore), poi mostra i messaggi dell'app.
# Ctrl+C per smettere di leggere i messaggi. Prima: .\compila.ps1
$ErrorActionPreference = 'Stop'
$adb = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
$apk = Join-Path $PSScriptRoot 'progetto\app\build\outputs\apk\debug\app-debug.apk'
# Identificativo provvisorio (app/build.gradle, applicationId).
$app = 'it.schedario.sviluppo'

$telefoni = & $adb devices | Select-Object -Skip 1 | Where-Object { $_ -match '\tdevice$' }
if (-not $telefoni) {
    Write-Host "Nessun telefono pronto. Collegalo con il cavo, attiva il debug USB e accetta la richiesta sullo schermo del telefono."
    & $adb devices -l
    exit 1
}
& $adb install -r $apk
& $adb shell am start -n "$app/it.schedario.app.MainActivity"
& $adb logcat -c
& $adb logcat -s schedario:V RustStdoutStderr:V GameActivity:V
