$ErrorActionPreference = "Continue"

Write-Host "=== 1. POST create ===" -ForegroundColor Cyan
$product = Invoke-RestMethod http://localhost:3000/products -Method Post -ContentType "application/json" -Body '{"name":"Laptop Pro","price":1299.99,"description":"High-end laptop","stock":10}'
$product | ConvertTo-Json
$id = $product.id
Write-Host "Captured ID: $id"

Write-Host "`n=== 2. GET list ===" -ForegroundColor Cyan
Invoke-RestMethod "http://localhost:3000/products" | ConvertTo-Json -Depth 4

Write-Host "`n=== 3. GET by id ===" -ForegroundColor Cyan
Invoke-RestMethod "http://localhost:3000/products/$id" | ConvertTo-Json

Write-Host "`n=== 4. PUT partial update (price+stock only) ===" -ForegroundColor Cyan
Invoke-RestMethod "http://localhost:3000/products/$id" -Method Put -ContentType "application/json" -Body '{"price":999.99,"stock":5}' | ConvertTo-Json

Write-Host "`n=== 5. POST 422 invalid payload ===" -ForegroundColor Cyan
try { Invoke-RestMethod http://localhost:3000/products -Method Post -ContentType "application/json" -Body '{"name":"","price":-1,"stock":-5}' } catch { Write-Host $_.ErrorDetails.Message }

Write-Host "`n=== 6. GET 404 unknown id ===" -ForegroundColor Cyan
try { Invoke-RestMethod "http://localhost:3000/products/00000000-0000-0000-0000-000000000000" } catch { Write-Host $_.ErrorDetails.Message }

Write-Host "`n=== 7. DELETE ===" -ForegroundColor Cyan
$r = Invoke-WebRequest "http://localhost:3000/products/$id" -Method Delete -UseBasicParsing
Write-Host "Status: $($r.StatusCode) (expect 204)"

Write-Host "`n=== 8. GET after delete → 404 ===" -ForegroundColor Cyan
try { Invoke-RestMethod "http://localhost:3000/products/$id" } catch { Write-Host $_.ErrorDetails.Message }
