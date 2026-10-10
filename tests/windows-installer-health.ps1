function Wait-ForHelperState([bool]$ExpectedHealthy) {
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            $response = Invoke-RestMethod 'http://127.0.0.1:8080/health' -TimeoutSec 1
            $healthy = $response.service -eq 'mendeley-loopback' -and $response.status -eq 'ok'
        } catch {
            $healthy = $false
        }
        if ($healthy -eq $ExpectedHealthy) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}
