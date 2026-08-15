# OD-006 browser secure DNS behavior probe.
# Non-elevated, read-only. Reports Chrome/Edge/Firefox secure DNS settings
# and infers whether .test would be resolved through Windows DNS client (NRPT)
# or through the browser's own DoH/secure DNS provider.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-OptionalProperty {
    param([object]$Object, [string]$Name, $Default = $null)
    if ($Object -and $Object.PSObject.Properties[$Name]) { return $Object.$Name }
    return $Default
}

$results = [System.Collections.Generic.List[object]]::new()

function Add-Result {
    param([string]$Browser, [string]$Setting, [string]$Value, [string]$Inference)
    $results.Add([PSCustomObject]@{
        Browser    = $Browser
        Setting    = $Setting
        Value      = $Value
        Inference  = $Inference
    })
}

# Chrome: Secure DNS settings are stored in the user profile preferences.
$chromePrefs = $null
$chromePaths = @(
    Join-Path $env:LOCALAPPDATA "Google\Chrome\User Data\Default\Secure Preferences"
    Join-Path $env:LOCALAPPDATA "Google\Chrome\User Data\Local State"
)
foreach ($path in $chromePaths) {
    if (Test-Path $path) {
        try {
            $chromePrefs = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -ErrorAction SilentlyContinue
            break
        }
        catch {
            $chromePrefs = $null
        }
    }
}

$chromeMode = Get-OptionalProperty (Get-OptionalProperty $chromePrefs "dns_over_https") "mode" "not configured"
$chromeTemplates = Get-OptionalProperty (Get-OptionalProperty $chromePrefs "dns_over_https") "templates" ""
$chromeInference = switch ($chromeMode) {
    "off"          { "NRPT applies; Windows DNS client resolves .test" }
    "automatic"    { "Browser may upgrade to known DoH providers; .test could bypass NRPT" }
    "secure"       { "Browser forces DoH to configured provider; .test bypasses NRPT" }
    default        { "Assuming Windows DNS client; verify manually" }
}
Add-Result -Browser "Chrome" -Setting "SecureDnsMode" -Value $chromeMode -Inference $chromeInference
if ($chromeTemplates) {
    Add-Result -Browser "Chrome" -Setting "SecureDnsTemplates" -Value ($chromeTemplates -join "; ") -Inference "DoH provider list"
}

# Edge: similar Chromium storage.
$edgePrefs = $null
$edgePaths = @(
    Join-Path $env:LOCALAPPDATA "Microsoft\Edge\User Data\Default\Secure Preferences"
    Join-Path $env:LOCALAPPDATA "Microsoft\Edge\User Data\Local State"
)
foreach ($path in $edgePaths) {
    if (Test-Path $path) {
        try {
            $edgePrefs = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -ErrorAction SilentlyContinue
            break
        }
        catch {
            $edgePrefs = $null
        }
    }
}

$edgeMode = Get-OptionalProperty (Get-OptionalProperty $edgePrefs "dns_over_https") "mode" "not configured"
$edgeTemplates = Get-OptionalProperty (Get-OptionalProperty $edgePrefs "dns_over_https") "templates" ""
$edgeInference = switch ($edgeMode) {
    "off"          { "NRPT applies; Windows DNS client resolves .test" }
    "automatic"    { "Browser may upgrade to known DoH providers; .test could bypass NRPT" }
    "secure"       { "Browser forces DoH to configured provider; .test bypasses NRPT" }
    default        { "Assuming Windows DNS client; verify manually" }
}
Add-Result -Browser "Edge" -Setting "SecureDnsMode" -Value $edgeMode -Inference $edgeInference
if ($edgeTemplates) {
    Add-Result -Browser "Edge" -Setting "SecureDnsTemplates" -Value ($edgeTemplates -join "; ") -Inference "DoH provider list"
}

# Firefox: network.trr.mode in profiles.ini / prefs.js.
$firefoxResult = "not installed or no profile found"
$firefoxProfileDir = Join-Path $env:APPDATA "Mozilla\Firefox\Profiles"
if (Test-Path $firefoxProfileDir) {
    $prefsFiles = Get-ChildItem -LiteralPath $firefoxProfileDir -Filter "prefs.js" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($prefsFiles) {
        $prefsPath = $prefsFiles.FullName
        $trrLine = Select-String -LiteralPath $prefsPath -Pattern 'user_pref\("network\.trr\.mode"' | Select-Object -Last 1
        if ($trrLine) {
            if ($trrLine.Line -match '\"network\.trr\.mode\"\s*,\s*(\d+)') {
                $trrMode = [int]$matches[1]
                $firefoxResult = $trrMode
                $trrInference = switch ($trrMode) {
                    0 { "Off; NRPT applies" }
                    1 { "Reserved/unknown" }
                    2 { "TRR first; .test likely bypasses NRPT" }
                    3 { "TRR only; .test bypasses NRPT" }
                    4 { "Reserved/unknown" }
                    5 { "TRR disabled; NRPT applies" }
                    default { "Unknown mode" }
                }
                Add-Result -Browser "Firefox" -Setting "network.trr.mode" -Value $trrMode -Inference $trrInference
            }
        }
        else {
            Add-Result -Browser "Firefox" -Setting "network.trr.mode" -Value "not set" -Inference "Default off; NRPT applies"
        }
    }
    else {
        Add-Result -Browser "Firefox" -Setting "network.trr.mode" -Value "no prefs.js" -Inference "Default off; NRPT applies"
    }
}
else {
    Add-Result -Browser "Firefox" -Setting "network.trr.mode" -Value "not installed" -Inference "N/A"
}

[PSCustomObject]@{
    GeneratedAt = (Get-Date).ToString("o")
    Machine     = $env:COMPUTERNAME
    User        = $env:USERNAME
    Results     = $results
} | ConvertTo-Json -Depth 5
