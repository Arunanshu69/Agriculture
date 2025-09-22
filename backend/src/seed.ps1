# -----------------------------
# Seed Script: Populate sample herbs (QR on-demand only)
# -----------------------------

param(
    [switch]$ResetDb
)

$baseUrl = "http://127.0.0.1:3000"
$Reset = [bool]$ResetDb
$qrFolder = "."

function Save-QR {
    param(
        [string]$qrBase64,
        [string]$fileName
    )
    if ([string]::IsNullOrEmpty($qrBase64)) { return }
    $qrBase64 = $qrBase64 -replace "^data:image/png;base64,", ""
    $qrBytes = [System.Convert]::FromBase64String($qrBase64)
    $qrPath = Join-Path $qrFolder $fileName
    [System.IO.File]::WriteAllBytes($qrPath, $qrBytes)
    Write-Host "Saved QR ->" $qrPath
}

$samples = @(
    # Global demo
    @{ name = "Mint";         farmer = "Alice";         location = "Farm A" },
    @{ name = "Basil";        farmer = "Bob";           location = "Farm B" },
    @{ name = "Tulsi";        farmer = "Ravi";          location = "Farm C" },
    @{ name = "Rosemary";     farmer = "Maya";          location = "Farm D" },
    @{ name = "Thyme";        farmer = "Noah";          location = "Farm E" },

    # India (Odisha) specific
    @{ name = "Tulsi";        farmer = "Sukanta";       location = "Bhubaneswar, Khordha, Odisha" },
    @{ name = "Neem";         farmer = "Priyanka";      location = "Cuttack, Odisha" },
    @{ name = "Coriander";    farmer = "Bikash";        location = "Puri, Odisha" },
    @{ name = "Curry Leaves"; farmer = "Nandita";       location = "Sambalpur, Odisha" },
    @{ name = "Moringa";      farmer = "Debasish";      location = "Berhampur, Ganjam, Odisha" },
    @{ name = "Ashwagandha";  farmer = "Chittaranjan";  location = "Rourkela, Sundargarh, Odisha" },
    @{ name = "Brahmi";       farmer = "Swati";         location = "Balasore, Odisha" },
    @{ name = "Lemongrass";   farmer = "Prakash";       location = "Kendrapara, Odisha" },
    @{ name = "Aloe Vera";    farmer = "Gitanjali";     location = "Koraput, Odisha" },
    @{ name = "Peppermint";   farmer = "Raghunath";     location = "Keonjhar, Odisha" },
    @{ name = "Fenugreek";    farmer = "Madhusmita";    location = "Mayurbhanj, Odisha" },
    @{ name = "Bay Leaf";     farmer = "Satyabrata";    location = "Kalahandi, Odisha" },
    @{ name = "Cumin";        farmer = "Swapna";        location = "Jajpur, Odisha" }
)

if ($Reset) {
    Write-Host "Resetting database..."
    try {
        Invoke-RestMethod -Uri "$baseUrl/resetDb" -Method Post -ErrorAction Stop | Out-Null
        Write-Host "Database reset done."
    } catch {
        Write-Host "Database reset failed: " $_
    }
}

Write-Host "Seeding" $samples.Count "herbs..."

foreach ($s in $samples) {
    $body = $s | ConvertTo-Json
    try {
        $resp = Invoke-RestMethod -Uri "$baseUrl/addHerb" -Method Post -Body $body -ContentType "application/json" -ErrorAction Stop
        $id = if ($resp.herb) { $resp.herb.id } else { $resp.id }
        if (-not $id) { $id = $resp.id }
        Write-Host "Added:" $id "(" $s.name ")"
    } catch {
        Write-Host "Failed to add" $s.name
        $r = $_.Exception.Response
        if ($r -and $r.GetResponseStream) {
            $reader = New-Object System.IO.StreamReader($r.GetResponseStream())
            $reader.ReadToEnd() | Write-Host
        }
    }
}

Write-Host "Listing all herbs after seeding..."
$list = Invoke-RestMethod -Uri "$baseUrl/listHerbs" -Method Get
foreach ($h in $list) {
    $id = if ($h.herb) { $h.herb.id } else { $h.id }
    $name = if ($h.herb) { $h.herb.name } else { $h.name }
    Write-Host "-" $id ":" $name
}

Write-Host "Done."