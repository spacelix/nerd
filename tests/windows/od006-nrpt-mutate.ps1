param(
    [string]$ResponderScript = (Join-Path $PSScriptRoot "od006-dns-responder.ps1"),
    [int]$DurationSeconds = 120,
    [switch]$TestSleepResume
)

# OD-006 NRPT mutation spike.
# Requires an elevated terminal. Adds a temporary .test -> 127.0.0.1 NRPT rule,
# runs the loopback responder, verifies UDP/TCP resolution, optionally pauses for
# a sleep/resume cycle, then removes the rule and verifies the exact rule set is
# restored. No other NRPT rule or DNS configuration is ever touched.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$spikeId = [Guid]::NewGuid().ToString("N").Substring(0, 8)
$ruleDisplayName = "Nerd OD-006 spike $spikeId"
$ruleComment = "nerd-od006-spike-$spikeId"
$results = [System.Collections.Generic.List[object]]::new()

function New-Result {
    param([string]$Step, [string]$Status, [string]$Detail)
    $results.Add([PSCustomObject]@{
        Step   = $Step
        Status = $Status
        Detail = $Detail
    })
}

$isElevated = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isElevated) {
    Write-Error "This spike must run from an elevated Windows terminal so NRPT can be mutated." -ErrorAction Continue
    exit 2
}

$ruleName = $null
$responder = $null
$snapshotBefore = @()

try {
    $snapshotBefore = @(Get-DnsClientNrptRule | ForEach-Object { $_.Name } | Sort-Object)
    New-Result "snapshot" "ok" ("{0} rules before" -f $snapshotBefore.Count)

    $udpListeners = @(Get-NetUDPEndpoint -LocalPort 53 -ErrorAction SilentlyContinue)
    $tcpListeners = @(Get-NetTCPConnection -LocalPort 53 -State Listen -ErrorAction SilentlyContinue)
    $portConflict = $udpListeners.Count -gt 0 -or $tcpListeners.Count -gt 0
    $portStatus = if ($portConflict) { "conflict" } else { "free" }
    New-Result "port-53" $portStatus ("udp=$($udpListeners.Count) tcp=$($tcpListeners.Count)")

    $added = Add-DnsClientNrptRule -Namespace ".test" -NameServers "127.0.0.1" -DisplayName $ruleDisplayName -Comment $ruleComment -PassThru
    $ruleName = $added.Name
    New-Result "add-rule" "ok" ("rule {0} added" -f $ruleName)

    $rule = Get-DnsClientNrptRule -Name $ruleName
    $namespaceOk = $rule.Namespace -contains ".test"
    $servers = @($rule.NameServers)
    $serversOk = $servers -contains "127.0.0.1"
    if (-not $namespaceOk -or -not $serversOk) {
        throw "added rule does not match expectations: namespace=$($rule.Namespace -join ',') servers=$($servers -join ',')"
    }
    New-Result "verify-rule" "ok" ".test -> 127.0.0.1"

    if (-not $portConflict) {
        $responderLog = Join-Path $env:TEMP "od006-responder-$spikeId.jsonl"
        $responderOut = Join-Path $env:TEMP "od006-responder-$spikeId.out.txt"
        $responderErr = Join-Path $env:TEMP "od006-responder-$spikeId.err.txt"
        $ps = [System.Diagnostics.Process]::GetCurrentProcess().MainModule.FileName
        $responder = Start-Process -FilePath $ps -ArgumentList @("-NoProfile", "-File", "`"$ResponderScript`"", "-DurationSeconds", "$DurationSeconds", "-LogPath", "`"$responderLog`"") -WindowStyle Hidden -PassThru -RedirectStandardOutput $responderOut -RedirectStandardError $responderErr
        Start-Sleep -Seconds 2
        New-Result "responder" "ok" ("pid {0}" -f $responder.Id)
    }
    else {
        New-Result "responder" "skipped" "port 53 occupied by a foreign listener; no responder started"
    }

    $viaNrpt = $null
    for ($attempt = 0; $attempt -lt 10; $attempt++) {
        try {
            $viaNrpt = Resolve-DnsName "foo.$spikeId.test" -DnsOnly -ErrorAction Stop | Where-Object { $_.Type -eq "A" } | Select-Object -First 1 -ExpandProperty IPAddress
        }
        catch {
            $viaNrpt = $null
        }
        if ($viaNrpt) { break }
        Start-Sleep -Milliseconds 500
    }
    $nrptStatus = if ($viaNrpt -eq "127.0.0.1") { "pass" } else { "fail" }
    $nrptDetail = if ($viaNrpt) { "resolved $viaNrpt" } else { "no A record" }
    New-Result "resolve-via-nrpt" $nrptStatus $nrptDetail

    if (-not $portConflict) {
        try {
            $direct = Resolve-DnsName "bar.$spikeId.test" -Server 127.0.0.1 -DnsOnly -ErrorAction Stop | Where-Object { $_.Type -eq "A" } | Select-Object -First 1 -ExpandProperty IPAddress
        }
        catch {
            $direct = $null
        }
        $directStatus = if ($direct -eq "127.0.0.1") { "pass" } else { "fail" }
        $directDetail = if ($direct) { "resolved $direct" } else { "no A record" }
        New-Result "resolve-direct" $directStatus $directDetail
    }
    else {
        New-Result "resolve-direct" "skipped" "responder not running"
    }

    if ($TestSleepResume) {
        Write-Host ""
        Write-Host "Suspend the machine now, resume it, then press Enter to re-test resolution."
        Read-Host
        $afterResume = $null
        try {
            $afterResume = Resolve-DnsName "resume.$spikeId.test" -DnsOnly -ErrorAction Stop | Where-Object { $_.Type -eq "A" } | Select-Object -First 1 -ExpandProperty IPAddress
        }
        catch {
            $afterResume = $null
        }
        $resumeStatus = if ($afterResume -eq "127.0.0.1") { "pass" } else { "fail" }
        $resumeDetail = if ($afterResume) { "resolved $afterResume" } else { "no A record" }
        New-Result "resolve-after-sleep-resume" $resumeStatus $resumeDetail
    }
}
catch {
    New-Result "fatal" "failed" $_.Exception.Message
}
finally {
    if ($responder -and -not $responder.HasExited) {
        Stop-Process -Id $responder.Id -Force -ErrorAction SilentlyContinue
        New-Result "responder-stop" "ok" ("pid {0} stopped" -f $responder.Id)
    }
    if ($ruleName) {
        Remove-DnsClientNrptRule -Name $ruleName -ErrorAction SilentlyContinue
        $removed = -not (Get-DnsClientNrptRule -Name $ruleName -ErrorAction SilentlyContinue)
        New-Result "remove-rule" $(if ($removed) { "ok" } else { "failed" }) ("rule {0} {1}" -f $ruleName, $(if ($removed) { "removed" } else { "still present" }))
    }
    $snapshotAfter = @(Get-DnsClientNrptRule | ForEach-Object { $_.Name } | Sort-Object)
    $diff = Compare-Object $snapshotBefore $snapshotAfter
    $restored = (-not $diff) -or ($diff.Count -eq 0)
    $restoreStatus = if ($restored) { "pass" } else { "fail" }
    New-Result "restore-verify" $restoreStatus ("{0} rules before, {1} rules after" -f $snapshotBefore.Count, $snapshotAfter.Count)
}

$report = [PSCustomObject]@{
    GeneratedAt = (Get-Date).ToString("o")
    Machine     = $env:COMPUTERNAME
    User        = $env:USERNAME
    SpikeId     = $spikeId
    RuleName    = $ruleName
    Results     = $results
}
$reportPath = Join-Path $env:TEMP "od006-mutate-latest.json"
$report | ConvertTo-Json -Depth 5 | Out-File -LiteralPath $reportPath -Encoding utf8
$report | ConvertTo-Json -Depth 5
