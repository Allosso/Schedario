# Raccoglie i testi di licenza di tutto cio' che finisce dentro Schedario e
# li scrive in assets\licenze\licenze.txt. Il programma li include e li mostra
# in Informazioni > Licenze; lo script per Windows li copia anche in dist.
#
# Perche': le licenze MIT, BSD, Apache-2.0 e simili chiedono di distribuire,
# insieme al programma, il testo della licenza e le righe di copyright di ogni
# libreria usata. I testi vengono dai pacchetti stessi (cartella di cargo),
# quindi sono quelli originali, con il copyright di ciascun autore.
#
# Va rieseguito quando cambiano le dipendenze (Cargo.lock o build.gradle):
#   .\script\genera-licenze.ps1
# Le librerie Java di Android (AndroidX, GameActivity) sono scritte sotto, a
# mano: se si aggiorna build.gradle, aggiornare anche $librerieJava.

$ErrorActionPreference = 'Continue'
$radice = Split-Path -Parent $PSScriptRoot
Set-Location $radice
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
[Threading.Thread]::CurrentThread.CurrentCulture = [Globalization.CultureInfo]::InvariantCulture

# Le piattaforme per cui si distribuisce Schedario.
$piattaforme = 'x86_64-pc-windows-gnu', 'x86_64-unknown-linux-gnu', 'aarch64-apple-darwin', 'x86_64-apple-darwin', 'aarch64-linux-android'
$propri = 'schedario', 'schedario-android'

$librerieJava = @(
    'androidx.games:games-activity 4.4.0 (GameActivity, GameTextInput)',
    'androidx.appcompat:appcompat 1.7.0 e le librerie AndroidX che richiede (activity, core, lifecycle, fragment, emoji2, ...)',
    'org.jetbrains.kotlin:kotlin-stdlib 1.8.22',
    'org.jetbrains.kotlinx:kotlinx-coroutines 1.6.4',
    'com.google.guava:listenablefuture 1.0',
    'org.jetbrains:annotations 13.0'
)

# 1. Quali pacchetti finiscono davvero nel programma, su ciascuna piattaforma.
$usati = @{}
foreach ($t in $piattaforme) {
    foreach ($riga in (cargo tree --workspace --locked --target $t -e normal --prefix none --format '{p}' 2>$null)) {
        if ($riga -match '^(\S+) v(\S+)') {
            $usati["$($Matches[1]) $($Matches[2])"] = $true
        }
    }
}
if ($usati.Count -eq 0) { throw 'cargo tree non ha restituito nulla' }

# 2. Dove stanno e che licenza dichiarano.
$metadati = (cargo metadata --format-version 1 --locked 2>$null) | ConvertFrom-Json
$pacchetti = $metadati.packages | Where-Object { $usati.ContainsKey("$($_.name) $($_.version)") -and $propri -notcontains $_.name } |
    Sort-Object name, version

# 3. I file di licenza di ciascuno, raggruppati per testo identico.
$schemaFile = '^(LICEN[CS]E|COPYING|COPYRIGHT|NOTICE|UNLICENSE|OFL|UFL|AUTHORS)'
$gruppi = [ordered]@{}
$senzaFile = @()
foreach ($p in $pacchetti) {
    $cartella = Split-Path $p.manifest_path -Parent
    $file = @(Get-ChildItem $cartella -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -match $schemaFile })
    # Caratteri e codice di altri inclusi nel pacchetto: licenze in una sottocartella.
    $file += @(Get-ChildItem $cartella -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch '^(src|tests|benches|examples|target)$' } |
        ForEach-Object { Get-ChildItem $_.FullName -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -match $schemaFile } })
    $nome = "$($p.name) $($p.version)"
    if ($file.Count -eq 0) {
        # Nessun file nel pacchetto: vale la licenza dichiarata; con piu'
        # licenze a scelta si usa MIT (o Apache-2.0). Il copyright e' degli
        # autori indicati dal pacchetto.
        $scelta = if ($p.license -match 'MIT') { 'MIT' } elseif ($p.license -match 'Apache-2\.0') { 'Apache-2.0' } else { $p.license }
        $autori = (($p.authors | ForEach-Object { ($_ -replace '\s*<[^>]*>', '').Trim() }) -join ', ')
        if (-not $autori) { $autori = "gli autori di $($p.name)" }
        $senzaFile += [pscustomobject]@{ Nome = $nome; Dichiarata = $p.license; Scelta = $scelta; Autori = $autori; Sito = $p.repository }
        continue
    }
    foreach ($f in $file | Sort-Object FullName) {
        $testo = ([IO.File]::ReadAllText($f.FullName) -replace "`r`n", "`n").Trim()
        if (-not $testo) { continue }
        if (-not $gruppi.Contains($testo)) { $gruppi[$testo] = New-Object System.Collections.Generic.List[string] }
        if (-not $gruppi[$testo].Contains($nome)) { $gruppi[$testo].Add($nome) }
    }
}

# 4. Il file.
$riga = '=' * 72
$s = New-Object System.Text.StringBuilder
function Add-Riga([string]$testo = '') { [void]$s.Append($testo).Append("`n") }

Add-Riga 'SCHEDARIO - LICENZE'
Add-Riga
Add-Riga 'Schedario e'' distribuito con la licenza MIT qui sotto. Contiene librerie'
Add-Riga 'e caratteri di altri autori, ciascuno con la sua licenza (tutte permissive):'
Add-Riga 'i testi completi seguono, ognuno con l''elenco di cio'' che lo usa.'
Add-Riga "Pacchetti Rust inclusi: $($pacchetti.Count). File generato da script\genera-licenze.ps1."
Add-Riga
Add-Riga $riga
Add-Riga 'Schedario'
Add-Riga $riga
Add-Riga ([IO.File]::ReadAllText("$radice\LICENSE") -replace "`r`n", "`n").Trim()
Add-Riga
Add-Riga $riga
Add-Riga 'Carattere Inter 4.1 (assets/caratteri), SIL Open Font License 1.1'
Add-Riga $riga
Add-Riga ([IO.File]::ReadAllText("$radice\assets\caratteri\Inter-LICENSE.txt") -replace "`r`n", "`n").Trim()
Add-Riga
Add-Riga $riga
Add-Riga 'Solo nella versione Android: librerie Java, licenza Apache-2.0'
Add-Riga '(testo completo della licenza Apache-2.0 piu'' sotto, nella sezione delle librerie Rust)'
Add-Riga $riga
foreach ($l in $librerieJava) { Add-Riga "- $l" }
Add-Riga
foreach ($testo in $gruppi.Keys) {
    Add-Riga $riga
    Add-Riga ('Usato da: ' + ($gruppi[$testo] -join ', '))
    Add-Riga $riga
    Add-Riga $testo
    Add-Riga
}
if ($senzaFile.Count) {
    Add-Riga $riga
    Add-Riga 'Pacchetti che non includono un file di licenza: vale la licenza che'
    Add-Riga 'dichiarano. Per MIT il testo e'' quello qui sotto, con il copyright degli'
    Add-Riga 'autori indicati per ciascuno; i testi di Apache-2.0, BSL-1.0 e Zlib sono'
    Add-Riga 'riportati sopra.'
    Add-Riga $riga
    foreach ($n in $senzaFile) {
        Add-Riga "- $($n.Nome): $($n.Scelta) (dichiarata: $($n.Dichiarata))"
        Add-Riga "  Copyright (c) $($n.Autori)"
        if ($n.Sito) { Add-Riga "  $($n.Sito)" }
    }
    Add-Riga
    Add-Riga 'Testo della licenza MIT:'
    Add-Riga
    foreach ($r in @(
        'Copyright (c) <gli autori indicati sopra>',
        '',
        'Permission is hereby granted, free of charge, to any person obtaining a copy',
        'of this software and associated documentation files (the "Software"), to deal',
        'in the Software without restriction, including without limitation the rights',
        'to use, copy, modify, merge, publish, distribute, sublicense, and/or sell',
        'copies of the Software, and to permit persons to whom the Software is',
        'furnished to do so, subject to the following conditions:',
        '',
        'The above copyright notice and this permission notice shall be included in all',
        'copies or substantial portions of the Software.',
        '',
        'THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR',
        'IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,',
        'FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE',
        'AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER',
        'LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,',
        'OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE',
        'SOFTWARE.')) { Add-Riga $r }
}

$uscita = "$radice\assets\licenze"
New-Item -ItemType Directory -Force $uscita | Out-Null
[IO.File]::WriteAllText("$uscita\licenze.txt", $s.ToString(), (New-Object System.Text.UTF8Encoding($false)))
$kb = [math]::Round((Get-Item "$uscita\licenze.txt").Length / 1KB)
Write-Host "Scritto assets\licenze\licenze.txt: $($pacchetti.Count) pacchetti, $($gruppi.Count) testi diversi, $kb KB"
if ($senzaFile.Count) { Write-Host "Senza file di licenza (testo standard + autori): $($senzaFile.Count)" }
