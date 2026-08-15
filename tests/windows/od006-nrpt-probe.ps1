param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,
    [switch]$PassThru
)

# OD-006 NRPT compatibility probe.
# Read-only: collects OS, NRPT, DNS client, DoH, interface, and service state.
# Never elevates, never mutates configuration, and is safe to run unprivileged.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Add-Result {
    param([string]$Block, [string]$Status, [object]$Data)
    [PSCustomObject]@{
        Block  = $Block
        Status = $Status
        Data   = $Data
    }
}

function Get-OptionalProperty {
    param([object]$InputObject, [string]$Name)
    $property = $InputObject.PSObject.Properties[$Name]
    if ($null -ne $property) { $property.Value } else { $null }
}

function ConvertTo-OptionalObject {
    param([object]$InputObject, [string[]]$Names)
    $result = [ordered]@{}
    foreach ($name in $Names) {
        $result[$name] = Get-OptionalProperty -InputObject $InputObject -Name $name
    }
    [PSCustomObject]$result
}

$results = [System.Collections.Generic.List[object]]::new()

# OS identity (OD-007 evidence).
try {
    $osBuild = [System.Environment]::OSVersion.Version
    $osInfo = [PSCustomObject]@{
        Caption      = (Get-CimInstance Win32_OperatingSystem).Caption
        Version      = $osBuild.ToString()
        Build        = "$($osBuild.Major).$($osBuild.Minor).$($osBuild.Build)"
        Architecture = (Get-CimInstance Win32_Processor).Architecture
        ProductType  = [System.Environment]::OSVersion.Platform
        Edition      = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").EditionID
        DisplayVersion = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").DisplayVersion
        CurrentBuild = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").CurrentBuild
    }
    $results.Add((Add-Result "os" "ok" $osInfo))
}
catch {
    $results.Add((Add-Result "os" "failed" $_.Exception.Message))
}

# NRPT rules (read-only).
try {
    $rules = @(Get-DnsClientNrptRule -ErrorAction Stop |
        ForEach-Object {
            $nameServers = Get-OptionalProperty -InputObject $_ -Name "NameServers"
            $optional = ConvertTo-OptionalObject -InputObject $_ -Names @(
                "Name", "DisplayName", "Namespace", "DnsSecEnable", "DAEnable",
                "IPsecRequired", "Comment"
            )
            [PSCustomObject]@{
                Name        = $optional.Name
                DisplayName = $optional.DisplayName
                Namespace   = $optional.Namespace
                NameServers = if ($nameServers) { $nameServers -join "," } else { $null }
                DnsSecEnable = $optional.DnsSecEnable
                DAEnable     = $optional.DAEnable
                IPsecRequired = $optional.IPsecRequired
                Comment      = $optional.Comment
            }
        })
    $results.Add((Add-Result "nrpt-rules" "ok" $rules))
}
catch {
    $results.Add((Add-Result "nrpt-rules" "failed" $_.Exception.Message))
}

# NRPT global state (read-only).
try {
    $global = Get-DnsClientNrptGlobal -ErrorAction Stop
    $optional = ConvertTo-OptionalObject -InputObject $global -Names @(
        "Enable", "QueryPolicy", "SecureNameQueryFallback"
    )
    $results.Add((Add-Result "nrpt-global" "ok" $optional))
}
catch {
    $results.Add((Add-Result "nrpt-global" "failed" $_.Exception.Message))
}

# DNS client server addresses per interface (read-only).
try {
    $servers = @(Get-DnsClientServerAddress -AddressFamily IPv4 -ErrorAction Stop |
        ForEach-Object {
            $addresses = @(Get-OptionalProperty -InputObject $_ -Name "ServerAddresses")
            if ($addresses.Count -eq 0) { return }
            [PSCustomObject]@{
                InterfaceAlias = $_.InterfaceAlias
                InterfaceIndex = $_.InterfaceIndex
                ServerAddresses = $addresses -join ","
            }
        })
    $results.Add((Add-Result "dns-servers" "ok" $servers))
}
catch {
    $results.Add((Add-Result "dns-servers" "failed" $_.Exception.Message))
}

# DoH server addresses and per-interface encryption settings (read-only).
try {
    $doh = @(Get-DnsClientDohServerAddress -ErrorAction Stop |
        ForEach-Object {
            [PSCustomObject]@{
                ServerAddress = $_.ServerAddress
                DohTemplate   = $_.DohTemplate
                AllowFallbackToUdp = $_.AllowFallbackToUdp
                AutoUpgrade   = $_.AutoUpgrade
            }
        })
    $results.Add((Add-Result "doh-servers" "ok" $doh))
}
catch {
    $results.Add((Add-Result "doh-servers" "unavailable" $_.Exception.Message))
}

try {
    $encryption = @(Get-DnsClientServerAddress -AddressFamily IPv4 -ErrorAction Stop |
        ForEach-Object {
            $props = Get-NetIPInterface -InterfaceIndex $_.InterfaceIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
            if (-not $props) { return }
            $templates = Get-OptionalProperty -InputObject $props -Name "DnsOverHttpsTemplates"
            [PSCustomObject]@{
                InterfaceAlias      = $_.InterfaceAlias
                InterfaceIndex      = $_.InterfaceIndex
                DnsOverHttpsTemplates = if ($templates) { $templates -join "," } else { $null }
                DnsOverHttpsEnabled   = Get-OptionalProperty -InputObject $props -Name "DnsOverHttpsEnabled"
                AdvertiseDefaultRoute = Get-OptionalProperty -InputObject $props -Name "AdvertiseDefaultRoute"
            }
        })
    $results.Add((Add-Result "doh-interface" "ok" $encryption))
}
catch {
    $results.Add((Add-Result "doh-interface" "unavailable" $_.Exception.Message))
}

# Active network interfaces and VPN presence (read-only).
try {
    $interfaces = @(Get-NetAdapter -ErrorAction Stop |
        Where-Object { $_.Status -eq "Up" } |
        ForEach-Object {
            $iface = Get-NetIPInterface -InterfaceIndex $_.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
            [PSCustomObject]@{
                Name          = $_.Name
                InterfaceIndex = $_.ifIndex
                MediaType     = $_.MediaType
                MacAddress    = $_.MacAddress
                LinkSpeed     = $_.LinkSpeed
                Virtual       = ($_.Virtual -eq $true)
                DnsOverHttpsEnabled = if ($iface) {
                    Get-OptionalProperty -InputObject $iface -Name "DnsOverHttpsEnabled"
                } else { $null }
            }
        })
    $results.Add((Add-Result "interfaces" "ok" $interfaces))
}
catch {
    $results.Add((Add-Result "interfaces" "failed" $_.Exception.Message))
}

# DNS client service state (read-only).
try {
    $service = Get-Service -Name Dnscache -ErrorAction Stop
    $results.Add((Add-Result "service-dnscache" "ok" ([PSCustomObject]@{
        Status = $service.Status
        StartType = $service.StartType
    })))
}
catch {
    $results.Add((Add-Result "service-dnscache" "failed" $_.Exception.Message))
}

# DNS-related policy presence from local policy/registry (read-only).
try {
    $policyPath = "HKLM:\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient"
    $policy = if (Test-Path $policyPath) {
        Get-ItemProperty $policyPath -ErrorAction Stop |
            Select-Object DohPolicy, DnsOverHttpsServers -ErrorAction SilentlyContinue
    } else {
        $null
    }
    $results.Add((Add-Result "policy-dnsclient" "ok" $policy))
}
catch {
    $results.Add((Add-Result "policy-dnsclient" "failed" $_.Exception.Message))
}

$summary = [PSCustomObject]@{
    GeneratedAt = (Get-Date).ToString("o")
    Machine     = $env:COMPUTERNAME
    User        = $env:USERNAME
    Elevated    = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
    Results     = $results
}

if ($PassThru) {
    $summary
}
else {
    $summary | ConvertTo-Json -Depth 6
}
