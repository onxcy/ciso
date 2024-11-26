$disk = (Get-Disk |
    Where-Object BusType -EQ 'USB' |
    Select-Object -Index 0 |
    Clear-Disk -RemoveData -RemoveOEM -PassThru |
    Initialize-Disk -PartitionStyle GPT -PassThru)
$ntfs = ($disk | New-Partition -Size 7GB | Format-Volume -FileSystem NTFS)
$uefi = ($disk | New-Partition -Size 1MB)
$iso = (Mount-DiskImage $iso_path -NoDriveLetter -PassThru | Get-Volume)
ConvertTo-Json @{
    ntfs = $ntfs
    iso  = $iso
    uefi = $uefi
}
