# Compila Schedario per Android: prima la libreria Rust con il compilatore
# dell'NDK (versione ottimizzata), poi l'app con Gradle (GameActivity e la
# parte Java).
#   .\compila.ps1                  versione di sviluppo:
#                                  progetto\app\build\outputs\apk\debug\app-debug.apk,
#                                  firmata con la chiave di sviluppo standard di Android
#   .\compila.ps1 -Pubblicazione   versione da pubblicare: il pacchetto per il
#                                  Play Store (bundle\release\app-release.aab) e l'APK
#                                  da distribuire altrove (apk\release\app-release.apk).
#                                  Servono schedarioId e il file della firma, vedi
#                                  progetto\app\build.gradle.
# Niente 'Stop': cargo e Gradle scrivono i progressi su stderr. Gli errori si
# controllano con $LASTEXITCODE.
param([switch]$Pubblicazione)
$ErrorActionPreference = 'Continue'

$env:JAVA_HOME = "$env:LOCALAPPDATA\Programs\jdk-21"
$env:ANDROID_HOME = "$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$ndk = "$env:ANDROID_HOME\ndk\28.2.13676358"
$env:ANDROID_NDK_ROOT = $ndk
$bin = "$ndk\toolchains\llvm\prebuilt\windows-x86_64\bin"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "$bin\aarch64-linux-android26-clang.cmd"
$env:CC_aarch64_linux_android = "$bin\aarch64-linux-android26-clang.cmd"
$env:CXX_aarch64_linux_android = "$bin\aarch64-linux-android26-clang++.cmd"
$env:AR_aarch64_linux_android = "$bin\llvm-ar.exe"
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:JAVA_HOME\bin;$env:PATH"
$gradle = "$env:LOCALAPPDATA\Programs\gradle-8.14.3\bin\gradle.bat"

$radice = Split-Path $PSScriptRoot -Parent
Push-Location $radice
try {
    cargo build --release --target aarch64-linux-android -p schedario-android
    if ($LASTEXITCODE -ne 0) { throw "Compilazione Rust non riuscita" }

    $librerie = "$PSScriptRoot\progetto\app\src\main\jniLibs\arm64-v8a"
    New-Item -ItemType Directory -Force $librerie | Out-Null
    Copy-Item "$radice\target\aarch64-linux-android\release\libschedario_android.so" $librerie -Force

    $uscite = "$PSScriptRoot\progetto\app\build\outputs"
    if ($Pubblicazione) {
        & $gradle -p "$PSScriptRoot\progetto" bundleRelease assembleRelease --console=plain
        if ($LASTEXITCODE -ne 0) { throw "Gradle non riuscito" }
        $risultati = "$uscite\bundle\release\app-release.aab", "$uscite\apk\release\app-release*.apk"
    } else {
        & $gradle -p "$PSScriptRoot\progetto" assembleDebug --console=plain
        if ($LASTEXITCODE -ne 0) { throw "Gradle non riuscito" }
        $risultati = "$uscite\apk\debug\app-debug.apk"
    }
    Get-Item $risultati | Select-Object FullName, @{ n = 'MB'; e = { [math]::Round($_.Length / 1MB, 1) } }
}
finally {
    Pop-Location
}
