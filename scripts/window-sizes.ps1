# Local Windows proof: resizes the real top-level window and captures that window only.
# Complements native-smoke.mjs, whose viewport sweep uses DevTools emulation.
#   powershell -ExecutionPolicy Bypass -File scripts/window-sizes.ps1 -OutDir .smoke/window
param([string]$Binary = "target/release/maddentool.exe", [string]$OutDir = ".smoke/window")

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int t, uint f);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[void][Win]::SetProcessDPIAware()
New-Item -ItemType Directory -Force $OutDir | Out-Null

$process = Start-Process -FilePath $Binary -PassThru
try {
  $deadline = (Get-Date).AddSeconds(30)
  while ($process.Refresh(), $process.MainWindowHandle -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 200
  }
  $handle = $process.MainWindowHandle
  if ($handle -eq [IntPtr]::Zero) { throw "The application window did not appear." }
  "title: $($process.MainWindowTitle)"

  foreach ($size in @(@(800, 600), @(1024, 768), @(1440, 900))) {
    [void][Win]::SetWindowPos($handle, [IntPtr]::Zero, 20, 20, $size[0], $size[1], 0x0040)
    Start-Sleep -Milliseconds 900
    $rect = New-Object Win+RECT
    [void][Win]::GetWindowRect($handle, [ref]$rect)
    $width = $rect.R - $rect.L
    $height = $rect.B - $rect.T
    $bitmap = New-Object System.Drawing.Bitmap $width, $height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    # PW_RENDERFULLCONTENT: reads the window itself, so other windows can never be captured.
    $dc = $graphics.GetHdc()
    $printed = [Win]::PrintWindow($handle, $dc, 2)
    $graphics.ReleaseHdc($dc)
    if (-not $printed) { throw "PrintWindow failed." }
    $file = Join-Path $OutDir "window-$($size[0])x$($size[1]).png"
    $bitmap.Save((Join-Path (Get-Location) $file))
    $graphics.Dispose()
    $bitmap.Dispose()
    "requested $($size[0])x$($size[1]) -> actual ${width}x${height}"
  }
} finally {
  Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
}
