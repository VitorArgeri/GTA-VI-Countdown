$ErrorActionPreference = "Stop"
$projectRoot = $PSScriptRoot
$hostInfo = rustc -Vv
if ($LASTEXITCODE -ne 0) { throw "Rust não está instalado ou não está no PATH." }
$hostLine = $hostInfo | Where-Object { $_ -like "host:*" } | Select-Object -First 1
if ($hostLine -notmatch "x86_64-pc-windows") {
    throw "Este empacotador exige a toolchain Windows x64; host encontrado: $hostLine"
}

Push-Location $projectRoot
try {
    cargo build --release --bin gta-vi-countdown -j 1
    if ($LASTEXITCODE -ne 0) { throw "A compilação de release falhou." }

    $exe = Join-Path $projectRoot "target\release\gta-vi-countdown.exe"
    $dist = Join-Path $projectRoot "dist"
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
        throw "Executável esperado não foi criado: $exe"
    }
    New-Item -ItemType Directory -Path $dist -Force | Out-Null
    Copy-Item -LiteralPath $exe -Destination $dist -Force
    $portableAssets = Join-Path $dist "assets"
    $portableAudio = Join-Path $portableAssets "audio"
    $portableWallpapers = Join-Path $portableAssets "wallpapers"
    New-Item -ItemType Directory -Path $portableAudio, $portableWallpapers -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $projectRoot "assets\audio\love-is-a-long-road.wav") `
        -Destination $portableAudio -Force
    Get-ChildItem -LiteralPath (Join-Path $projectRoot "assets\wallpapers") -File |
        ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination $portableWallpapers -Force }
    Copy-Item -LiteralPath (Join-Path $projectRoot "assets\icon.ico") -Destination $portableAssets -Force
    Copy-Item -LiteralPath (Join-Path $projectRoot "assets\icon.png") -Destination $portableAssets -Force
    Write-Output "Distribuição Windows x64 atualizada em $dist"
}
finally {
    Pop-Location
}
