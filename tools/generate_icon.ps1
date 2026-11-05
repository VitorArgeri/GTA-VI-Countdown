$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$assetFolder = Join-Path (Split-Path $PSScriptRoot -Parent) "assets"
New-Item -ItemType Directory -Path $assetFolder -Force | Out-Null
$master = [System.Drawing.Bitmap]::new(256, 256, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [System.Drawing.Graphics]::FromImage($master)
$graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
$graphics.Clear([System.Drawing.Color]::Transparent)

$shape = [System.Drawing.Drawing2D.GraphicsPath]::new()
$shape.AddArc(12, 12, 64, 64, 180, 90)
$shape.AddArc(180, 12, 64, 64, 270, 90)
$shape.AddArc(180, 180, 64, 64, 0, 90)
$shape.AddArc(12, 180, 64, 64, 90, 90)
$shape.CloseFigure()
$backdrop = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(0, 0, 256, 256),
    [System.Drawing.Color]::FromArgb(255, 20, 24, 36),
    [System.Drawing.Color]::FromArgb(255, 48, 31, 55),
    [System.Drawing.Drawing2D.LinearGradientMode]::ForwardDiagonal
)
$graphics.FillPath($backdrop, $shape)
$outline = [System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(115, 232, 137, 178), 3)
$graphics.DrawPath($outline, $shape)

$font = [System.Drawing.Font]::new("Segoe UI", 112, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
$format = [System.Drawing.StringFormat]::new()
$format.Alignment = [System.Drawing.StringAlignment]::Center
$format.LineAlignment = [System.Drawing.StringAlignment]::Center
$letterFill = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(32, 46, 192, 150),
    [System.Drawing.Color]::FromArgb(255, 244, 145, 188),
    [System.Drawing.Color]::FromArgb(255, 122, 215, 207),
    [System.Drawing.Drawing2D.LinearGradientMode]::Horizontal
)
$graphics.DrawString("VI", $font, $letterFill, [System.Drawing.RectangleF]::new(24, 33, 208, 168), $format)
$accent = [System.Drawing.Drawing2D.LinearGradientBrush]::new(
    [System.Drawing.Rectangle]::new(66, 204, 124, 4),
    [System.Drawing.Color]::FromArgb(255, 232, 137, 178),
    [System.Drawing.Color]::FromArgb(255, 119, 207, 200),
    [System.Drawing.Drawing2D.LinearGradientMode]::Horizontal
)
$graphics.FillRectangle($accent, 66, 204, 124, 4)
$master.Save((Join-Path $assetFolder "icon.png"), [System.Drawing.Imaging.ImageFormat]::Png)

$sizes = @(16, 24, 32, 48, 64, 128, 256)
$frames = foreach ($size in $sizes) {
    $frame = [System.Drawing.Bitmap]::new($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $frameGraphics = [System.Drawing.Graphics]::FromImage($frame)
    $frameGraphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $frameGraphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $frameGraphics.DrawImage($master, 0, 0, $size, $size)
    $stream = [System.IO.MemoryStream]::new()
    $frame.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
    [pscustomobject]@{ Size = $size; Bytes = $stream.ToArray() }
    $stream.Dispose()
    $frameGraphics.Dispose()
    $frame.Dispose()
}

$iconPath = Join-Path $assetFolder "icon.ico"
$file = [System.IO.File]::Create($iconPath)
$writer = [System.IO.BinaryWriter]::new($file)
$writer.Write([UInt16]0)
$writer.Write([UInt16]1)
$writer.Write([UInt16]$frames.Count)
$offset = 6 + 16 * $frames.Count
foreach ($frame in $frames) {
    $edge = if ($frame.Size -eq 256) { [byte]0 } else { [byte]$frame.Size }
    $writer.Write($edge)
    $writer.Write($edge)
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]32)
    $writer.Write([UInt32]$frame.Bytes.Length)
    $writer.Write([UInt32]$offset)
    $offset += $frame.Bytes.Length
}
foreach ($frame in $frames) { $writer.Write([byte[]]$frame.Bytes) }
$writer.Dispose()
$file.Dispose()
$graphics.Dispose()
$master.Dispose()
$shape.Dispose()
$backdrop.Dispose()
$outline.Dispose()
$font.Dispose()
$format.Dispose()
$letterFill.Dispose()
$accent.Dispose()
Write-Output "Ícones PNG e ICO atualizados em $assetFolder"
