param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("Create", "Remove")]
    [string]$Action,
    [Parameter(Mandatory = $true)]
    [ValidatePattern("^NerdF01_[0-9a-f]{8}$")]
    [string]$UserName,
    [string]$PasswordFile
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "temporary-user helper requires elevation"
}

$description = "Nerd Feature 01 temporary cross-user fixture"

if ($Action -eq "Create") {
    if ($null -ne (Get-LocalUser -Name $UserName -ErrorAction SilentlyContinue)) {
        throw "temporary user already exists"
    }
    if ([string]::IsNullOrWhiteSpace($PasswordFile) -or
        -not (Test-Path -LiteralPath $PasswordFile)) {
        throw "temporary user DPAPI password file was not supplied"
    }
    $password = Get-Content -LiteralPath $PasswordFile -Raw | ConvertTo-SecureString
    $created = $false
    try {
        New-LocalUser `
            -Name $UserName `
            -Password $password `
            -AccountNeverExpires `
            -PasswordNeverExpires `
            -Description $description | Out-Null
        $created = $true
        $usersGroupSid = New-Object Security.Principal.SecurityIdentifier("S-1-5-32-545")
        $usersGroup = $usersGroupSid.Translate([Security.Principal.NTAccount]).Value.Split("\")[-1]
        $user = Get-LocalUser -Name $UserName
        $alreadyMember = Get-LocalGroupMember -Group $usersGroup |
            Where-Object { $_.SID -eq $user.Sid }
        if ($null -eq $alreadyMember) {
            Add-LocalGroupMember -Group $usersGroup -Member $UserName
        }
    }
    catch {
        if ($created) {
            Remove-LocalUser -Name $UserName -ErrorAction SilentlyContinue
        }
        throw
    }
    exit 0
}

$user = Get-LocalUser -Name $UserName -ErrorAction SilentlyContinue
if ($null -eq $user) {
    exit 0
}
if ($user.Description -ne $description) {
    throw "refusing to remove an account without the Nerd test marker"
}
$sid = $user.Sid.Value
Remove-LocalUser -Name $UserName
$profile = Get-CimInstance Win32_UserProfile -Filter "SID='$sid'" -ErrorAction SilentlyContinue
if ($null -ne $profile) {
    $profile | Remove-CimInstance
}
