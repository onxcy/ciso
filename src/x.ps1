$disk = (Get-Disk | Where-Object -Property BusType -EQ 'USB' | Select-Object -Index 0)
if ($null -eq $disk) { exit 1 }
$disk | Clear-Disk -RemoveData -RemoveOEM -confirm:$false -PassThru | Initialize-Disk -PartitionStyle GPT | Out-Null
$size = $disk.Size * 0.95
$disk | New-Partition -Size $size -AssignDriveLetter | Format-Volume -FileSystem NTFS | Out-Null
($disk | New-Partition -Size 1MB).AccessPaths