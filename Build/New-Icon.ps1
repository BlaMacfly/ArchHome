<#
.SYNOPSIS
    Génère les images du launcher depuis Launcher\assets\HomeRPG-original.png : logo-512.png et logo-256.png
    (carrés, fond transparent) et homerpg.ico (16 à 256 px, images PNG), utilisée par Launcher\build.rs.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$assets = Join-Path (Split-Path -Parent $PSScriptRoot) 'Launcher\assets'
$src = [System.Drawing.Image]::FromFile((Join-Path $assets 'HomeRPG-original.png'))

function New-Square([int]$Size) {
    $bmp = New-Object System.Drawing.Bitmap $Size, $Size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    # Image d'origine centrée, proportions conservées.
    $scale = [Math]::Min($Size / $src.Width, $Size / $src.Height)
    $w = [int]($src.Width * $scale); $h = [int]($src.Height * $scale)
    $g.DrawImage($src, [int](($Size - $w) / 2), [int](($Size - $h) / 2), $w, $h)
    $g.Dispose()
    return $bmp
}

foreach ($s in 512, 256) {
    $b = New-Square $s
    $b.Save((Join-Path $assets "logo-$s.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
}

# Icône multi-résolution : en-tête ICONDIR puis une entrée par taille, chaque image en PNG.
$sizes = 16, 24, 32, 48, 64, 128, 256
$images = foreach ($s in $sizes) {
    $b = New-Square $s
    $ms = New-Object System.IO.MemoryStream
    $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $b.Dispose()
    , $ms.ToArray()
}
$out = New-Object System.IO.MemoryStream
$w = New-Object System.IO.BinaryWriter $out
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]
    $dim = if ($s -ge 256) { 0 } else { $s }
    $w.Write([byte]$dim); $w.Write([byte]$dim); $w.Write([byte]0); $w.Write([byte]0)
    $w.Write([uint16]1); $w.Write([uint16]32)
    $w.Write([uint32]$images[$i].Length); $w.Write([uint32]$offset)
    $offset += $images[$i].Length
}
foreach ($img in $images) { $w.Write($img) }
$w.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $assets 'homerpg.ico'), $out.ToArray())
$src.Dispose()
Write-Host "logo-512.png, logo-256.png, homerpg.ico -> $assets"
