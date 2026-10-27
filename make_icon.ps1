$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# PortZap: đĩa xanh lá + tia sấm (zap) trắng.
$Sizes = @(16, 24, 32, 48, 64, 128, 256)
$dir = Join-Path $PSScriptRoot 'icons'
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }

$greenTop = [System.Drawing.Color]::FromArgb(255, 34, 197, 94)
$greenBot = [System.Drawing.Color]::FromArgb(255, 21, 128, 61)
$ringCol  = [System.Drawing.Color]::FromArgb(255, 6, 95, 70)

# Zap (Feather-style) trong viewBox 24x24, tâm (12,12).
$Bolt = @(
  @{ x = 13;  y = 2 },
  @{ x = 3;   y = 14 },
  @{ x = 12;  y = 14 },
  @{ x = 11;  y = 22 },
  @{ x = 21;  y = 10 },
  @{ x = 12;  y = 10 }
)

function New-IconBitmap([int]$s) {
  $bmp = New-Object System.Drawing.Bitmap($s, $s, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  try {
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $pad = [math]::Max(0.5, $s * 0.02)
    $body = New-Object System.Drawing.RectangleF($pad, $pad, ($s - 2 * $pad), ($s - 2 * $pad))
    $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($body, $greenTop, $greenBot, 90)
    try { $g.FillEllipse($brush, $body) } finally { $brush.Dispose() }

    $ringW = [math]::Max(1.0, $s * 0.04)
    $inset = $pad + ($ringW / 2)
    $ringRect = New-Object System.Drawing.RectangleF($inset, $inset, ($s - 2 * $inset), ($s - 2 * $inset))
    $ringPen = New-Object System.Drawing.Pen($ringCol, $ringW)
    try { $g.DrawEllipse($ringPen, $ringRect) } finally { $ringPen.Dispose() }

    $k = $s / 24.0 * 0.88
    $pts = @(
      foreach ($p in $Bolt) {
        [System.Drawing.PointF]::new(
          [float]($s / 2 + ($p.x - 12) * $k),
          [float]($s / 2 + ($p.y - 12) * $k)
        )
      }
    )
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddPolygon([System.Drawing.PointF[]]$pts)
    $g.FillPath([System.Drawing.Brushes]::White, $path)
    $boltPen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, ([float][math]::Max(0.6, $s * 0.022)))
    $boltPen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    try { $g.DrawPath($boltPen, $path) } finally { $boltPen.Dispose() }
    $path.Dispose()
  }
  finally {
    $g.Dispose()
  }
  return $bmp
}

function Get-Bgra([System.Drawing.Bitmap]$bmp) {
  $w = $bmp.Width
  $h = $bmp.Height
  $rect = New-Object System.Drawing.Rectangle(0, 0, $w, $h)
  $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  try {
    $stride = $data.Stride
    $raw = New-Object 'byte[]' ($stride * $h)
    [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $raw, 0, $raw.Length)
  }
  finally {
    $bmp.UnlockBits($data)
  }
  if ($stride -eq $w * 4) { return , $raw }
  $tight = New-Object 'byte[]' ($w * $h * 4)
  for ($y = 0; $y -lt $h; $y++) {
    [Array]::Copy($raw, $y * $stride, $tight, $y * $w * 4, $w * 4)
  }
  return , $tight
}

$frames = @()
foreach ($s in $Sizes) {
  $bmp = New-IconBitmap $s
  $frames += , @{ size = $s; px = (Get-Bgra $bmp) }
  if ($s -eq 256) {
    $bmp.Save((Join-Path $dir 'icon.png'), [System.Drawing.Imaging.ImageFormat]::Png)
  }
  $bmp.Dispose()
}

$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
$bw.Write([uint16]0)
$bw.Write([uint16]1)
$bw.Write([uint16]$frames.Count)

$offset = 6 + (16 * $frames.Count)
foreach ($f in $frames) {
  $s = $f.size
  $andStride = [math]::Ceiling($s / 32.0) * 4
  $entrySize = 40 + ($s * $s * 4) + ($andStride * $s)
  $b = if ($s -ge 256) { [byte]0 } else { [byte]$s }
  $bw.Write($b)
  $bw.Write($b)
  $bw.Write([byte]0)
  $bw.Write([byte]0)
  $bw.Write([uint16]1)
  $bw.Write([uint16]32)
  $bw.Write([uint32]$entrySize)
  $bw.Write([uint32]$offset)
  $offset += $entrySize
}

foreach ($f in $frames) {
  $s = $f.size
  $px = $f.px
  $andStride = [math]::Ceiling($s / 32.0) * 4
  $bw.Write([uint32]40)
  $bw.Write([int32]$s)
  $bw.Write([int32]($s * 2))
  $bw.Write([uint16]1)
  $bw.Write([uint16]32)
  $bw.Write([uint32]0)
  $bw.Write([uint32]($s * $s * 4))
  $bw.Write([int32]0)
  $bw.Write([int32]0)
  $bw.Write([uint32]0)
  $bw.Write([uint32]0)
  for ($y = $s - 1; $y -ge 0; $y--) { $bw.Write($px, $y * $s * 4, $s * 4) }
  $bw.Write((New-Object 'byte[]' ($andStride * $s)))
}
$bw.Flush()

$ico = Join-Path $dir 'icon.ico'
[IO.File]::WriteAllBytes($ico, $ms.ToArray())
Write-Host "icon.ico written: $($ms.Length) bytes ($($Sizes -join ', '))"
Write-Host "icon.png written (256px preview)"
