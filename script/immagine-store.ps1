# Crea l'immagine in evidenza del Play Store (1024 x 500) in
# dist\play-store\immagine-in-evidenza.png: l'icona dell'app a sinistra, il
# nome e una riga di descrizione a destra, nei colori del programma e con il
# carattere Inter del progetto (licenza OFL, assets\caratteri).
# Uso (cartella Note):  .\script\immagine-store.ps1
# Usa System.Drawing di Windows; l'icona la disegna examples\icona_store.rs.

$ErrorActionPreference = 'Stop'
$radice = Split-Path -Parent $PSScriptRoot
Set-Location $radice
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:LOCALAPPDATA\Programs\mingw64\bin;$env:Path"
Add-Type -AssemblyName System.Drawing

$uscita = Join-Path $radice 'dist\play-store'
New-Item -ItemType Directory -Force $uscita | Out-Null
$icona = Join-Path $uscita 'icona-angoli.png'
$ErrorActionPreference = 'Continue'
cargo run --release --quiet --example icona_store -- $icona --angoli | Out-Null
$ErrorActionPreference = 'Stop'
if (-not (Test-Path $icona)) { throw 'Icona non creata' }

$caratteri = New-Object System.Drawing.Text.PrivateFontCollection
$caratteri.AddFontFile((Join-Path $radice 'assets\caratteri\Inter-SemiBold.ttf'))
$caratteri.AddFontFile((Join-Path $radice 'assets\caratteri\Inter-Regular.ttf'))
$famiglia = $caratteri.Families[0]

$immagine = New-Object System.Drawing.Bitmap 1024, 500
$g = [System.Drawing.Graphics]::FromImage($immagine)
$g.SmoothingMode = 'AntiAlias'
$g.TextRenderingHint = 'AntiAliasGridFit'
$g.InterpolationMode = 'HighQualityBicubic'

# Fondo color carta, come le note; in basso una fascia nel colore principale.
$g.Clear([System.Drawing.Color]::FromArgb(0xFA, 0xF8, 0xF4))
$g.FillRectangle((New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(0x2F, 0x6B, 0x7E))), 0, 420, 1024, 80)

$disegno = [System.Drawing.Image]::FromFile($icona)
$g.DrawImage($disegno, 80, 60, 320, 320)
$disegno.Dispose()

$carta = [System.Drawing.Color]::FromArgb(0xFA, 0xF8, 0xF4)
$accento = [System.Drawing.Color]::FromArgb(0x2F, 0x6B, 0x7E)
$grigio = [System.Drawing.Color]::FromArgb(0x4A, 0x58, 0x5E)
$g.DrawString('Schedario', (New-Object System.Drawing.Font($famiglia, 88, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)),
    (New-Object System.Drawing.SolidBrush $accento), 440, 140)
$descrizione = New-Object System.Drawing.Font($famiglia, 30, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)
$g.DrawString('Note a schede, sincronizzate', $descrizione, (New-Object System.Drawing.SolidBrush $grigio), 446, 262)
$g.DrawString('e cifrate in rete locale', $descrizione, (New-Object System.Drawing.SolidBrush $grigio), 446, 302)
$chiaro = $carta
$punto = [string][char]0x00B7   # il file resta ASCII: PowerShell 5.1 lo leggerebbe male
$g.DrawString("Nessun account  $punto  nessun cloud", (New-Object System.Drawing.Font($famiglia, 24, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)),
    (New-Object System.Drawing.SolidBrush $chiaro), 80, 446)

$file = Join-Path $uscita 'immagine-in-evidenza.png'
$immagine.Save($file, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $immagine.Dispose()
Remove-Item $icona
Write-Host "Pronta: $file"
