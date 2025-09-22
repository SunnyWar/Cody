# Self-play driver for the Cody engine (PowerShell)
# Starts from initial position and alternates moves by querying the engine via UCI
# Usage: .\self_play.ps1 -Plies 10 -Depth 2
param(
    [int]$Plies = 10,
    [int]$Depth = 2
)

$enginePath = Join-Path $PSScriptRoot '..\target\debug\engine.exe'
if (-not (Test-Path $enginePath)) {
    Write-Error "Engine binary not found at $enginePath; build the project first"
    exit 1
}

# Start engine as a process with redirected stdin/stdout
$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $enginePath
$startInfo.RedirectStandardInput = $true
$startInfo.RedirectStandardOutput = $true
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true

$proc = New-Object System.Diagnostics.Process
$proc.StartInfo = $startInfo
$proc.Start() | Out-Null
$stdin = $proc.StandardInput
$stdout = $proc.StandardOutput

# Initialize UCI
$stdin.WriteLine('uci')
# read until uciok
while ($null -ne ($line = $stdout.ReadLine())) {
    Write-Host $line
    if ($line -eq 'uciok') { break }
}

$stdin.WriteLine('isready')
while ($null -ne ($line = $stdout.ReadLine())) {
    Write-Host $line
    if ($line -eq 'readyok') { break }
}

$stdin.WriteLine('ucinewgame')

$moves = @()
for ($ply = 0; $ply -lt $Plies; $ply++) {
    $posCmd = 'position startpos' + ($(if ($moves.Count -gt 0) { ' moves ' + ($moves -join ' ') } else { '' }))
    $stdin.WriteLine($posCmd)
    $stdin.WriteLine("go depth $Depth")

    # Read until bestmove line
    $best = $null
    while ($null -ne ($line = $stdout.ReadLine())) {
        Write-Host $line
        if ($line.StartsWith('bestmove')) {
            $parts = $line.Split(' ')
            $best = $parts[1]
            break
        }
    }

    if (($best -eq '0000') -or ($best -eq '000')) {
        Write-Host "Engine returned null move or no move, stopping"
        break
    }

    $moves += $best
}

# Cleanup
$stdin.WriteLine('quit')
$proc.WaitForExit()
Write-Host "Self-play finished; moves: $($moves -join ' ')"