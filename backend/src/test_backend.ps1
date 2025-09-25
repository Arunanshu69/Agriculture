# -----------------------------
# Backend Test Script with QR Save (Robust Version)
# -----------------------------

$baseUrl = "http://127.0.0.1:3000"
$qrFolder = "."

# Create folder if it doesn't exist
if (-not (Test-Path $qrFolder)) {
    New-Item -ItemType Directory -Path $qrFolder | Out-Null
}

# Helper function to save QR code
function Save-QR {
    param(
        [string]$qrBase64,
        [string]$fileName
    )
    if ([string]::IsNullOrEmpty($qrBase64)) {
        Write-Host "Warning: QR code data is empty. Skipping save for $fileName"
        return
    }
    $qrBase64 = $qrBase64 -replace "^data:image/png;base64,", ""
    $qrBytes = [System.Convert]::FromBase64String($qrBase64)
    $qrPath = Join-Path $qrFolder $fileName
    [System.IO.File]::WriteAllBytes($qrPath, $qrBytes)
    Write-Host "QR Code saved at:" $qrPath
}

# -----------------------------
# 0️⃣ Reset Database (for clean test run)
# -----------------------------
Write-Host "Resetting database for a clean test run..."
try {
    Invoke-RestMethod -Uri "$baseUrl/resetDb" -Method Post -ErrorAction Stop | Out-Null
    Write-Host "Reset successful."
} catch {
    Write-Host "Warning: Reset may have failed or endpoint unavailable. Continuing..."
}

# Quick sanity check: list should be empty
try {
    $initialList = Invoke-RestMethod -Uri "$baseUrl/listHerbs" -Method Get -ErrorAction Stop
    $count = if ($initialList) { $initialList.Count } else { 0 }
    Write-Host "Herbs after reset:" $count
} catch {
    Write-Host "Could not verify list after reset."
}

# -----------------------------
# 1️⃣ Add Herb
# -----------------------------
$addHerbBody = @{
    name = "Blue Spider Lily"
    farmer = "Muzan Kibutsuji"
    location = "Kyoto, Japan"
} | ConvertTo-Json

Write-Host "Adding herb..."
try {
    $addResponse = Invoke-RestMethod -Uri "$baseUrl/addHerb" -Method Post -Body $addHerbBody -ContentType "application/json" -ErrorAction Stop

    # Debug: print full response
    Write-Host "Full addResponse:"
    $addResponse | ConvertTo-Json -Depth 5

    # Extract herb ID safely
    if ($addResponse -and $addResponse.PSObject.Properties['herb']) {
        $herbId = $addResponse.herb.id
    } elseif ($addResponse -and $addResponse.PSObject.Properties['id']) {
        $herbId = $addResponse.id
    } else {
        Write-Host "Error: Unable to find herb ID in response"
        exit 1
    }
} catch {
    Write-Host "Failed to add herb"
    $resp = $_.Exception.Response
    if ($resp -and $resp.GetResponseStream) {
        try {
            $reader = New-Object System.IO.StreamReader($resp.GetResponseStream())
            $body = $reader.ReadToEnd()
            if ($body) { Write-Host "Server response:" $body }
        } catch {}
    }
    exit 1
}

Write-Host "Added herb ID:" $herbId
Write-Host "(QR generation is on-demand; skipping QR on add)"

# -----------------------------
# 2️⃣ Get Herb by ID
# -----------------------------
Write-Host "`nGetting herb by ID..."
try {
    $getResponse = Invoke-RestMethod -Uri "$baseUrl/getHerb/$herbId" -Method Get
    Write-Host "Herb retrieved:" $getResponse.herb.name
    Save-QR -qrBase64 $getResponse.qr_code -fileName "$herbId-get.png"
} catch {
    Write-Host "Error fetching herb by ID. It may not exist."
}

# -----------------------------
# 3️⃣ Get printable QR PNG via /qr/{id}
# -----------------------------
Write-Host "`nDownloading QR PNG via /qr/{id}..."
try {
    $qrPngPath = "$herbId-qr.png"
    Invoke-WebRequest -Uri "$baseUrl/qr/$herbId" -OutFile $qrPngPath -ErrorAction Stop | Out-Null
    Write-Host "Saved QR PNG ->" $qrPngPath
} catch {
    Write-Host "Error downloading QR PNG."
}

# -----------------------------
# 4️⃣ Public product endpoint /p/{id}
# -----------------------------
Write-Host "`nFetching public product JSON via /p/{id}..."
try {
    $publicResp = Invoke-RestMethod -Uri "$baseUrl/p/$herbId" -Method Get -ErrorAction Stop
    Write-Host "Public product name:" $publicResp.name
} catch {
    Write-Host "Error fetching public product endpoint."
}

# -----------------------------
# 5️⃣ Scan endpoint with different payloads
# -----------------------------
Write-Host "`nTesting /scan endpoint..."
try {
    $scanPlain = @{ data = $herbId } | ConvertTo-Json
    $scanUrl = @{ data = "$baseUrl/p/$herbId" } | ConvertTo-Json
    $innerJson = @{ id = $herbId } | ConvertTo-Json -Compress
    $scanJsonPayload = @{ data = $innerJson } | ConvertTo-Json

    $respPlain = Invoke-RestMethod -Uri "$baseUrl/scan" -Method Post -Body $scanPlain -ContentType "application/json"
    Write-Host "Scan plain id ->" $respPlain.name

    $respUrl = Invoke-RestMethod -Uri "$baseUrl/scan" -Method Post -Body $scanUrl -ContentType "application/json"
    Write-Host "Scan URL ->" $respUrl.name

    $respJson = Invoke-RestMethod -Uri "$baseUrl/scan" -Method Post -Body $scanJsonPayload -ContentType "application/json"
    Write-Host "Scan JSON payload ->" $respJson.name
} catch {
    Write-Host "Error testing /scan endpoint."
}

# -----------------------------
# 6️⃣ Update Herb and verify
# -----------------------------
Write-Host "`nUpdating herb via PUT /updateHerb/{id}..."
try {
    $updateBody = @{ name = "Blue Spider Lily (Updated)"; location = "Kyoto, Japan" } | ConvertTo-Json
    $updateResp = Invoke-RestMethod -Uri "$baseUrl/updateHerb/$herbId" -Method Put -Body $updateBody -ContentType "application/json"
    Write-Host "Updated name:" $updateResp.name "| location:" $updateResp.location

    Write-Host "Verifying update via /p/{id}..."
    $verify = Invoke-RestMethod -Uri "$baseUrl/p/$herbId" -Method Get
    Write-Host "Verify name:" $verify.name "| location:" $verify.location
} catch {
    Write-Host "Error updating/verifying herb."
}

# -----------------------------
# 7️⃣ List All Herbs
# -----------------------------
Write-Host "`nListing all herbs..."
$listResponse = Invoke-RestMethod -Uri "$baseUrl/listHerbs" -Method Get
foreach ($herb in $listResponse) {
    $id = if ($herb.herb) { $herb.herb.id } else { $herb.id }
    $name = if ($herb.herb) { $herb.herb.name } else { $herb.name }
    $farmer = if ($herb.herb) { $herb.herb.farmer } else { $herb.farmer }

    Write-Host "ID:" $id ", Name:" $name ", Farmer:" $farmer
    # QR not generated in list view
}
Write-Host "List completed."

# -----------------------------
# 8️⃣ Delete Herb by ID
# -----------------------------
Write-Host "`nDeleting herb..."
try {
    $deleteResponse = Invoke-RestMethod -Uri "$baseUrl/deleteHerb/$herbId" -Method Delete
    Write-Host $deleteResponse
} catch {
    Write-Host "Error deleting herb."
}

# -----------------------------
# 9️⃣ Verify Deletion
# -----------------------------
Write-Host "`nVerifying deletion..."
try {
    Invoke-RestMethod -Uri "$baseUrl/getHerb/$herbId" -Method Get
    Write-Host "Herb still exists! Deletion may have failed."
} catch {
    Write-Host "Herb not found. Deletion successful!"
}