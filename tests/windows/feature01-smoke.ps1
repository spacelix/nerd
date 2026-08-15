param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path,
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug",
    [switch]$AllowTemporaryLocalUser,
    [switch]$ValidateElevatedRejection
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$daemon = Join-Path $RepoRoot "target\$Configuration\nerd-daemon.exe"
$cli = Join-Path $RepoRoot "target\$Configuration\nerd.exe"
$dataRoot = Join-Path $env:LOCALAPPDATA "Nerd"
$pipeName = "\\.\pipe\Nerd.Control.6f843fb5-1bc8-4d47-a038-751c7c218fe8"
$mutexName = "Global\Nerd.Daemon.6f843fb5-1bc8-4d47-a038-751c7c218fe8"
$temporaryUserHelper = Join-Path $PSScriptRoot "feature01-temp-user.ps1"
$crossUserProbe = Join-Path $PSScriptRoot "feature01-cross-user-probe.ps1"
$fixtureOwned = $false
$cleanupSucceeded = $false

if (-not (Test-Path -LiteralPath $daemon)) {
    throw "daemon binary not found: $daemon"
}
if (-not (Test-Path -LiteralPath $cli)) {
    throw "CLI binary not found: $cli"
}
if (Test-Path -LiteralPath $dataRoot) {
    throw "clean fixture required; refusing to modify existing Nerd data"
}
if ($AllowTemporaryLocalUser -and
    (-not (Test-Path -LiteralPath $temporaryUserHelper) -or
     -not (Test-Path -LiteralPath $crossUserProbe))) {
    throw "cross-user helper files are missing"
}

$nativeSource = @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class NerdFeature01Native
{
    public const uint CREATE_NEW_PROCESS_GROUP = 0x00000200;
    public const uint CREATE_NO_WINDOW = 0x08000000;
    public const uint LOGON_WITH_PROFILE = 1;
    public const uint CTRL_BREAK_EVENT = 1;
    public const uint WAIT_OBJECT_0 = 0;
    public const uint WAIT_TIMEOUT = 258;
    public const uint STILL_ACTIVE = 259;
    public const uint GENERIC_READ = 0x80000000;
    public const uint GENERIC_WRITE = 0x40000000;
    public const uint OPEN_EXISTING = 3;
    public const uint DACL_SECURITY_INFORMATION = 0x00000004;
    public const int SE_KERNEL_OBJECT = 6;

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct STARTUPINFO
    {
        public uint cb;
        public string lpReserved;
        public string lpDesktop;
        public string lpTitle;
        public uint dwX;
        public uint dwY;
        public uint dwXSize;
        public uint dwYSize;
        public uint dwXCountChars;
        public uint dwYCountChars;
        public uint dwFillAttribute;
        public uint dwFlags;
        public ushort wShowWindow;
        public ushort cbReserved2;
        public IntPtr lpReserved2;
        public IntPtr hStdInput;
        public IntPtr hStdOutput;
        public IntPtr hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESS_INFORMATION
    {
        public IntPtr hProcess;
        public IntPtr hThread;
        public uint dwProcessId;
        public uint dwThreadId;
    }

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool CreateProcessW(
        string applicationName,
        string commandLine,
        IntPtr processAttributes,
        IntPtr threadAttributes,
        [MarshalAs(UnmanagedType.Bool)] bool inheritHandles,
        uint creationFlags,
        IntPtr environment,
        string currentDirectory,
        ref STARTUPINFO startupInfo,
        out PROCESS_INFORMATION processInformation);

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool CreateProcessWithLogonW(
        string userName,
        string domain,
        string password,
        uint logonFlags,
        string applicationName,
        StringBuilder commandLine,
        uint creationFlags,
        IntPtr environment,
        string currentDirectory,
        ref STARTUPINFO startupInfo,
        out PROCESS_INFORMATION processInformation);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GenerateConsoleCtrlEvent(uint controlEvent, uint processGroupId);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetExitCodeProcess(IntPtr process, out uint exitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool TerminateProcess(IntPtr process, uint exitCode);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool CloseHandle(IntPtr handle);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr CreateFileW(
        string fileName,
        uint desiredAccess,
        uint shareMode,
        IntPtr securityAttributes,
        uint creationDisposition,
        uint flagsAndAttributes,
        IntPtr templateFile);

    [DllImport("advapi32.dll", SetLastError = true)]
    public static extern uint GetSecurityInfo(
        IntPtr handle,
        int objectType,
        uint securityInfo,
        IntPtr owner,
        IntPtr group,
        IntPtr dacl,
        IntPtr sacl,
        out IntPtr securityDescriptor);

    [DllImport("advapi32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool ConvertSecurityDescriptorToStringSecurityDescriptorW(
        IntPtr securityDescriptor,
        uint requestedStringSDRevision,
        uint securityInformation,
        out IntPtr stringSecurityDescriptor,
        out uint stringSecurityDescriptorLength);

    [DllImport("kernel32.dll")]
    public static extern IntPtr LocalFree(IntPtr memory);
}
"@

Add-Type -TypeDefinition $nativeSource -Language CSharp

function Invoke-NerdStatus {
    $previousPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = (& $cli status 2>&1 | Out-String).Trim()
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousPreference
    }
    [PSCustomObject]@{
        ExitCode = $exitCode
        Output = $output
    }
}

function Assert-NerdPipeAcl {
    $pipe = [NerdFeature01Native]::CreateFileW(
        $pipeName,
        [NerdFeature01Native]::GENERIC_READ -bor [NerdFeature01Native]::GENERIC_WRITE,
        0,
        [IntPtr]::Zero,
        [NerdFeature01Native]::OPEN_EXISTING,
        0,
        [IntPtr]::Zero)
    if ($pipe -eq [IntPtr](-1)) {
        throw "CreateFileW for pipe ACL failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    }

    $descriptor = [IntPtr]::Zero
    $sddlPointer = [IntPtr]::Zero
    try {
        $result = [NerdFeature01Native]::GetSecurityInfo(
            $pipe,
            [NerdFeature01Native]::SE_KERNEL_OBJECT,
            [NerdFeature01Native]::DACL_SECURITY_INFORMATION,
            [IntPtr]::Zero,
            [IntPtr]::Zero,
            [IntPtr]::Zero,
            [IntPtr]::Zero,
            [ref]$descriptor)
        if ($result -ne 0) {
            throw "GetSecurityInfo failed: $result"
        }

        $length = 0
        $converted = [NerdFeature01Native]::ConvertSecurityDescriptorToStringSecurityDescriptorW(
            $descriptor,
            1,
            [NerdFeature01Native]::DACL_SECURITY_INFORMATION,
            [ref]$sddlPointer,
            [ref]$length)
        if (-not $converted) {
            throw "security descriptor conversion failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
        }
        $sddl = [Runtime.InteropServices.Marshal]::PtrToStringUni($sddlPointer)
        $userSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        if (-not $sddl.StartsWith("D:P")) {
            throw "pipe DACL is not protected: $sddl"
        }
        if ([regex]::Matches($sddl, "\(A;").Count -ne 2) {
            throw "pipe DACL must contain exactly two allow ACEs: $sddl"
        }
        if (-not $sddl.Contains(";;;SY)") -or -not $sddl.Contains(";;;$userSid)")) {
            throw "pipe DACL must allow only LocalSystem and current user: $sddl"
        }
        if ($sddl.Contains(";;;WD)") -or $sddl.Contains(";;;AN)") -or $sddl.Contains(";;;BA)")) {
            throw "pipe DACL grants a broader principal: $sddl"
        }
    }
    finally {
        if ($sddlPointer -ne [IntPtr]::Zero) {
            [NerdFeature01Native]::LocalFree($sddlPointer) | Out-Null
        }
        if ($descriptor -ne [IntPtr]::Zero) {
            [NerdFeature01Native]::LocalFree($descriptor) | Out-Null
        }
        [NerdFeature01Native]::CloseHandle($pipe) | Out-Null
    }
}

function Test-NerdFixtureShape {
    if (-not (Test-Path -LiteralPath $dataRoot)) {
        return $true
    }
    $rootEntries = @(Get-ChildItem -LiteralPath $dataRoot -Force)
    if (@($rootEntries | Where-Object { $_.Name -notin @("nerd.db", "logs") }).Count -ne 0) {
        return $false
    }
    $logRoot = Join-Path $dataRoot "logs"
    if (Test-Path -LiteralPath $logRoot) {
        $unexpectedLogs = @(
            Get-ChildItem -LiteralPath $logRoot -Force |
                Where-Object { $_.PSIsContainer -or $_.Name -notlike "nerd-daemon.jsonl*" }
        )
        if ($unexpectedLogs.Count -ne 0) {
            return $false
        }
    }
    return $true
}

function New-RandomTestPassword {
    $bytes = New-Object byte[] 24
    $generator = New-Object Security.Cryptography.RNGCryptoServiceProvider
    try {
        $generator.GetBytes($bytes)
    }
    finally {
        $generator.Dispose()
    }
    return ([Convert]::ToBase64String($bytes) + "aA1!")
}

function Invoke-TemporaryUserAction {
    param(
        [ValidateSet("Create", "Remove")]
        [string]$Action,
        [string]$UserName,
        [string]$Password
    )

    $powershell = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    $passwordFile = $null
    try {
        if ($Action -eq "Create") {
            $passwordFile = Join-Path $env:LOCALAPPDATA (
                "NerdF01-password-$([Guid]::NewGuid().ToString('N')).txt")
            $securePassword = ConvertTo-SecureString $Password -AsPlainText -Force
            $encryptedPassword = ConvertFrom-SecureString $securePassword
            [IO.File]::WriteAllText(
                $passwordFile,
                $encryptedPassword,
                (New-Object Text.UTF8Encoding($false)))
        }
        $arguments = @(
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-File", "`"$temporaryUserHelper`"",
            "-Action", $Action,
            "-UserName", $UserName
        )
        if ($Action -eq "Create") {
            $arguments += @("-PasswordFile", "`"$passwordFile`"")
        }
        $result = Start-Process `
            -FilePath $powershell `
            -ArgumentList $arguments `
            -Verb RunAs `
            -Wait `
            -PassThru
        if ($result.ExitCode -ne 0) {
            throw "temporary-user $Action failed with exit code $($result.ExitCode)"
        }
    }
    finally {
        if ($null -ne $passwordFile -and (Test-Path -LiteralPath $passwordFile)) {
            Remove-Item -LiteralPath $passwordFile -Force
        }
    }
}

function Assert-CrossUserPipeDenied {
    param(
        [string]$UserName,
        [string]$Password
    )

    $powershell = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    $commandLine = New-Object Text.StringBuilder(
        "`"$powershell`" -NoProfile -ExecutionPolicy Bypass -File `"$crossUserProbe`" -PipeName `"$pipeName`"")
    $startup = New-Object NerdFeature01Native+STARTUPINFO
    $startup.cb = [Runtime.InteropServices.Marshal]::SizeOf($startup)
    $probe = New-Object NerdFeature01Native+PROCESS_INFORMATION
    $started = [NerdFeature01Native]::CreateProcessWithLogonW(
        $UserName,
        ".",
        $Password,
        [NerdFeature01Native]::LOGON_WITH_PROFILE,
        $powershell,
        $commandLine,
        [NerdFeature01Native]::CREATE_NO_WINDOW,
        [IntPtr]::Zero,
        $RepoRoot,
        [ref]$startup,
        [ref]$probe)
    if (-not $started) {
        throw "CreateProcessWithLogonW failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    }

    try {
        [NerdFeature01Native]::CloseHandle($probe.hThread) | Out-Null
        $probe.hThread = [IntPtr]::Zero
        $wait = [NerdFeature01Native]::WaitForSingleObject($probe.hProcess, 10000)
        if ($wait -ne [NerdFeature01Native]::WAIT_OBJECT_0) {
            [NerdFeature01Native]::TerminateProcess($probe.hProcess, 99) | Out-Null
            throw "cross-user probe did not exit before deadline"
        }
        $exitCode = 0
        if (-not [NerdFeature01Native]::GetExitCodeProcess($probe.hProcess, [ref]$exitCode)) {
            throw "cross-user GetExitCodeProcess failed"
        }
        if ($exitCode -ne 0) {
            throw "cross-user pipe probe failed with exit code $exitCode"
        }
    }
    finally {
        if ($probe.hProcess -ne [IntPtr]::Zero) {
            [NerdFeature01Native]::CloseHandle($probe.hProcess) | Out-Null
        }
        if ($probe.hThread -ne [IntPtr]::Zero) {
            [NerdFeature01Native]::CloseHandle($probe.hThread) | Out-Null
        }
    }
    Write-Output "Cross-user pipe access: denied."
}

$before = Invoke-NerdStatus
if ($before.ExitCode -ne 3) {
    throw "expected absent daemon exit 3, got $($before.ExitCode): $($before.Output)"
}

if ($ValidateElevatedRejection) {
    $elevated = Start-Process `
        -FilePath $daemon `
        -WorkingDirectory $RepoRoot `
        -Verb RunAs `
        -Wait `
        -PassThru
    if ($elevated.ExitCode -ne 14) {
        throw "expected elevated daemon rejection exit 14, got $($elevated.ExitCode)"
    }
    if (Test-Path -LiteralPath $dataRoot) {
        throw "elevated daemon mutated the data directory before rejection"
    }
    Write-Output "Elevated daemon token: rejected before mutation."
}

$startup = New-Object NerdFeature01Native+STARTUPINFO
$startup.cb = [Runtime.InteropServices.Marshal]::SizeOf($startup)
$process = New-Object NerdFeature01Native+PROCESS_INFORMATION
$started = [NerdFeature01Native]::CreateProcessW(
    $daemon,
    $null,
    [IntPtr]::Zero,
    [IntPtr]::Zero,
    $false,
    [NerdFeature01Native]::CREATE_NEW_PROCESS_GROUP,
    [IntPtr]::Zero,
    $RepoRoot,
    [ref]$startup,
    [ref]$process)
if (-not $started) {
    throw "CreateProcessW failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
}

$gracefulExit = $false
try {
    [NerdFeature01Native]::CloseHandle($process.hThread) | Out-Null
    $process.hThread = [IntPtr]::Zero

    $status = $null
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        $status = Invoke-NerdStatus
        if ($status.ExitCode -eq 0) {
            break
        }
        if ($status.ExitCode -ne 3) {
            throw "unexpected status exit $($status.ExitCode): $($status.Output)"
        }
        Start-Sleep -Milliseconds 50
    }
    if ($null -eq $status -or $status.ExitCode -ne 0) {
        throw "daemon did not become ready: $($status.Output)"
    }
    $pidMatch = [regex]::Match($status.Output, "(?m)^PID: ([0-9]+)\r?$")
    if (-not $pidMatch.Success -or [uint32]$pidMatch.Groups[1].Value -ne $process.dwProcessId) {
        throw "status PID does not match spawned daemon PID $($process.dwProcessId)"
    }
    $fixtureOwned = $true
    Write-Output $status.Output
    $workingSetMatch = [regex]::Match($status.Output, "Working set: ([0-9.]+) MiB")
    if (-not $workingSetMatch.Success) {
        throw "working-set metric missing from status output"
    }
    $workingSetMiB = [double]::Parse(
        $workingSetMatch.Groups[1].Value,
        [Globalization.CultureInfo]::InvariantCulture)
    if ($workingSetMiB -ge 20.0) {
        throw "daemon working set exceeds 20 MiB budget: $workingSetMiB MiB"
    }

    Assert-NerdPipeAcl

    if ($AllowTemporaryLocalUser) {
        $temporaryUserName = "NerdF01_$([Guid]::NewGuid().ToString('N').Substring(0, 8))"
        $temporaryPassword = New-RandomTestPassword
        $temporaryUserCreated = $false
        try {
            Invoke-TemporaryUserAction `
                -Action Create `
                -UserName $temporaryUserName `
                -Password $temporaryPassword
            $temporaryUserCreated = $true
            Assert-CrossUserPipeDenied `
                -UserName $temporaryUserName `
                -Password $temporaryPassword
        }
        finally {
            if ($temporaryUserCreated) {
                Invoke-TemporaryUserAction -Action Remove -UserName $temporaryUserName
            }
            $temporaryPassword = $null
        }
    }

    Start-Sleep -Seconds 1
    $daemonProcess = Get-Process -Id $process.dwProcessId
    $cpuStart = $daemonProcess.TotalProcessorTime.TotalMilliseconds
    Start-Sleep -Seconds 2
    $daemonProcess.Refresh()
    $idleCpuMilliseconds =
        $daemonProcess.TotalProcessorTime.TotalMilliseconds - $cpuStart
    if ($idleCpuMilliseconds -gt 50.0) {
        throw "daemon used $idleCpuMilliseconds ms CPU during 2-second idle sample"
    }
    Write-Output ("Idle CPU (2 seconds): {0:F1} ms" -f $idleCpuMilliseconds)

    $previousPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $secondOutput = (& $daemon 2>&1 | Out-String).Trim()
        $secondExit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousPreference
    }
    if ($secondExit -ne 10) {
        throw "expected second daemon exit 10, got ${secondExit}: $secondOutput"
    }

    $shutdownTimer = [Diagnostics.Stopwatch]::StartNew()
    if (-not [NerdFeature01Native]::GenerateConsoleCtrlEvent(
        [NerdFeature01Native]::CTRL_BREAK_EVENT,
        $process.dwProcessId)) {
        throw "GenerateConsoleCtrlEvent failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    }

    $wait = [NerdFeature01Native]::WaitForSingleObject($process.hProcess, 4500)
    $shutdownTimer.Stop()
    if ($wait -ne [NerdFeature01Native]::WAIT_OBJECT_0) {
        throw "daemon did not stop before deadline; wait result $wait"
    }
    if ($shutdownTimer.Elapsed.TotalMilliseconds -ge 4000.0) {
        throw "daemon exceeded the four-second shutdown deadline: $($shutdownTimer.Elapsed.TotalMilliseconds) ms"
    }
    $exitCode = 0
    if (-not [NerdFeature01Native]::GetExitCodeProcess($process.hProcess, [ref]$exitCode)) {
        throw "GetExitCodeProcess failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
    }
    if ($exitCode -ne 0) {
        throw "daemon exited with $exitCode"
    }
    Write-Output ("Graceful shutdown: {0:F1} ms" -f $shutdownTimer.Elapsed.TotalMilliseconds)
    $gracefulExit = $true

    $after = Invoke-NerdStatus
    if ($after.ExitCode -ne 3) {
        throw "expected absent daemon after shutdown, got $($after.ExitCode): $($after.Output)"
    }
    Write-Output "Feature 01 process smoke test passed."
}
finally {
    if ($process.hProcess -ne [IntPtr]::Zero) {
        $exitCode = 0
        if ([NerdFeature01Native]::GetExitCodeProcess($process.hProcess, [ref]$exitCode) -and
            $exitCode -eq [NerdFeature01Native]::STILL_ACTIVE) {
            [NerdFeature01Native]::TerminateProcess($process.hProcess, 99) | Out-Null
            [NerdFeature01Native]::WaitForSingleObject($process.hProcess, 5000) | Out-Null
        }
        [NerdFeature01Native]::CloseHandle($process.hProcess) | Out-Null
    }
    if ($process.hThread -ne [IntPtr]::Zero) {
        [NerdFeature01Native]::CloseHandle($process.hThread) | Out-Null
    }
    if ($fixtureOwned) {
        $cleanupMutex = $null
        $ownsCleanupMutex = $false
        try {
            $createdNew = $false
            $cleanupMutex = [Threading.Mutex]::new($false, $mutexName, [ref]$createdNew)
            try {
                $ownsCleanupMutex = $cleanupMutex.WaitOne(0)
            }
            catch [Threading.AbandonedMutexException] {
                $ownsCleanupMutex = $true
            }
            if ($ownsCleanupMutex) {
                $cleanupStatus = Invoke-NerdStatus
                $liveDaemon = Get-Process -Name "nerd-daemon" -ErrorAction SilentlyContinue
                if ($cleanupStatus.ExitCode -eq 3 -and
                    $null -eq $liveDaemon -and
                    (Test-NerdFixtureShape)) {
                    if (Test-Path -LiteralPath $dataRoot) {
                        Remove-Item -LiteralPath $dataRoot -Recurse -Force
                    }
                    $cleanupSucceeded = $true
                }
            }
        }
        finally {
            if ($ownsCleanupMutex) {
                $cleanupMutex.ReleaseMutex()
            }
            if ($null -ne $cleanupMutex) {
                $cleanupMutex.Dispose()
            }
        }
    }
    else {
        $cleanupSucceeded = -not (Test-Path -LiteralPath $dataRoot)
    }
}

if (-not $gracefulExit) {
    throw "daemon required forced cleanup"
}
if (-not $cleanupSucceeded) {
    throw "fixture cleanup could not prove exclusive ownership; Nerd data was preserved"
}
