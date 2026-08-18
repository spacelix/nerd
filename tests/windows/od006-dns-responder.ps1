param(
    [int]$DurationSeconds = 120,
    [string]$LogPath,
    [int]$UdpPort = 53,
    [int]$TcpPort = 53
)

# OD-006 spike loopback DNS responder.
# Binds 127.0.0.1 on UDP and TCP and answers A queries only for .test names
# with 127.0.0.1. Non-.test names receive NXDOMAIN. Used only during the
# compatibility spike; never part of the production daemon.

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-LogLine {
    param([string]$Level, [string]$Message)
    $line = [PSCustomObject]@{
        timestamp = (Get-Date).ToString("o")
        level     = $Level
        message   = $Message
    } | ConvertTo-Json -Compress
    if ($LogPath) {
        Add-Content -LiteralPath $LogPath -Value $line
    }
    else {
        Write-Output $line
    }
}

function Decode-DnsName {
    param([byte[]]$Message, [ref]$Offset)
    $labels = [System.Collections.Generic.List[string]]::new()
    $end = $Message.Length
    while ($true) {
        if ($Offset.Value -ge $end) { return $null }
        $length = [int]$Message[$Offset.Value]
        $Offset.Value++
        if ($length -eq 0) { break }
        if (($length -band 0xC0) -eq 0xC0) { return $null }
        if ($length -gt 63 -or ($Offset.Value + $length) -gt $end) { return $null }
        $labels.Add([System.Text.Encoding]::ASCII.GetString($Message, $Offset.Value, $length))
        $Offset.Value += $length
    }
    ($labels -join ".").ToLowerInvariant()
}

function New-DnsResponse {
    param([byte[]]$Query)
    if ($Query.Length -lt 12) { return $null }
    $id = $Query[0..1]
    $qdcount = [BitConverter]::ToUInt16(@($Query[5], $Query[4]), 0)
    if ($qdcount -eq 0) { return $null }

    $offset = 12
    $name = Decode-DnsName -Message $Query -Offset ([ref]$offset)
    if ($null -eq $name -or $offset + 4 -gt $Query.Length) { return $null }

    $isTest = $name -eq "test" -or $name.EndsWith(".test")
    $question = $Query[12..($offset + 3)]
    if (-not $isTest) {
        $flags = @(0x81, 0x83)
        $answerCount = 0
        $response = @( $id[0]; $id[1]; $flags[0]; $flags[1]; $Query[4]; $Query[5];
                      0; 0; 0; 0; 0; 0 )
        $response += $question
        return [byte[]]$response
    }

    $flags = @(0x81, 0x80)
    $answerCount = 1
    $answer = @(
        0xC0, 0x0C,          # pointer to question name
        0x00, 0x01,          # type A
        0x00, 0x01,          # class IN
        0x00, 0x00, 0x00, 0x3C,  # TTL 60
        0x00, 0x04,          # rdlength
        127, 0, 0, 1         # rdata 127.0.0.1
    )
    $response = @( $id[0]; $id[1]; $flags[0]; $flags[1]; $Query[4]; $Query[5];
                   [byte]$answerCount; 0; 0; 0; 0; 0 )
    $response += $question
    $response += $answer
    [byte[]]$response
}

function Invoke-UdpResponder {
    param([int]$Port)
    $udp = [System.Net.Sockets.UdpClient]::new([System.Net.IPAddress]::Loopback, $Port)
    try {
        $remote = [System.Net.IPEndPoint]::new([System.Net.IPAddress]::Any, 0)
        while ($true) {
            $bytes = $udp.Receive([ref]$remote)
            $response = New-DnsResponse -Query $bytes
            if ($response) {
                $udp.Send($response, $response.Length, $remote) | Out-Null
            }
        }
    }
    finally {
        $udp.Close()
    }
}

function Invoke-TcpResponder {
    param([int]$Port)
    $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, $Port)
    $listener.Start()
    try {
        while ($true) {
            $client = $listener.AcceptTcpClient()
            try {
                $stream = $client.GetStream()
                $lengthBuffer = New-Object byte[] 2
                if ($stream.Read($lengthBuffer, 0, 2) -ne 2) { continue }
                $length = [BitConverter]::ToUInt16(@($lengthBuffer[1], $lengthBuffer[0]), 0)
                if ($length -eq 0 -or $length -gt 4096) { continue }
                $query = New-Object byte[] $length
                $read = 0
                while ($read -lt $length) {
                    $n = $stream.Read($query, $read, $length - $read)
                    if ($n -le 0) { break }
                    $read += $n
                }
                if ($read -ne $length) { continue }
                $response = New-DnsResponse -Query $query
                if ($response) {
                    $outLength = [BitConverter]::GetBytes([UInt16]$response.Length)
                    $stream.WriteByte($outLength[1])
                    $stream.WriteByte($outLength[0])
                    $stream.Write($response, 0, $response.Length)
                    $stream.Flush()
                }
            }
            finally {
                $client.Close()
            }
        }
    }
    finally {
        $listener.Stop()
    }
}

Write-LogLine "start" "responder starting on 127.0.0.1 udp=$UdpPort tcp=$TcpPort duration=$DurationSeconds"

$script:udpPort = $UdpPort
$script:tcpPort = $TcpPort
$udpTask = [System.Threading.Tasks.Task]::Run([Action]{ Invoke-UdpResponder -Port $script:udpPort })
$tcpTask = [System.Threading.Tasks.Task]::Run([Action]{ Invoke-TcpResponder -Port $script:tcpPort })

$deadline = (Get-Date).AddSeconds($DurationSeconds)
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
}

Write-LogLine "stop" "duration elapsed; responder stopping"
$udpTask.Wait(2000) | Out-Null
$tcpTask.Wait(2000) | Out-Null
Write-LogLine "stop" "responder stopped"
