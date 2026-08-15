param(
    [Parameter(Mandatory = $true)]
    [string]$PipeName
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$source = @"
using System;
using System.Runtime.InteropServices;

public static class NerdCrossUserProbeNative
{
    public const uint GENERIC_READ = 0x80000000;
    public const uint GENERIC_WRITE = 0x40000000;
    public const uint OPEN_EXISTING = 3;
    public const int ERROR_ACCESS_DENIED = 5;

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr CreateFileW(
        string fileName,
        uint desiredAccess,
        uint shareMode,
        IntPtr securityAttributes,
        uint creationDisposition,
        uint flagsAndAttributes,
        IntPtr templateFile);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool CloseHandle(IntPtr handle);
}
"@

Add-Type -TypeDefinition $source -Language CSharp
$pipe = [NerdCrossUserProbeNative]::CreateFileW(
    $PipeName,
    [NerdCrossUserProbeNative]::GENERIC_READ -bor [NerdCrossUserProbeNative]::GENERIC_WRITE,
    0,
    [IntPtr]::Zero,
    [NerdCrossUserProbeNative]::OPEN_EXISTING,
    0,
    [IntPtr]::Zero)
if ($pipe -ne [IntPtr](-1)) {
    [NerdCrossUserProbeNative]::CloseHandle($pipe) | Out-Null
    exit 41
}
if ([Runtime.InteropServices.Marshal]::GetLastWin32Error() -eq
    [NerdCrossUserProbeNative]::ERROR_ACCESS_DENIED) {
    exit 0
}
exit 42
