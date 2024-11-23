use std::{ffi::OsStr, process::Command};

use serde::Deserialize;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, GENERIC_WRITE},
        Storage::FileSystem::{
            CreateFileW, WriteFile, FILE_FLAG_NO_BUFFERING, FILE_FLAG_WRITE_THROUGH,
            FILE_SHARE_NONE, OPEN_EXISTING,
        },
    },
};

const KILO: usize = 1024;
const MEGA: usize = 1024 * KILO;
const GIGA: usize = 1024 * MEGA;

fn main() {
    let disk = get_disk().pop().expect("No USB device found.");
    println!("Using disk {} \"{}\"", disk.id, disk.name);

    clear_disk(&disk);
    init_disk(&disk);

    part_disk(&disk, 7 * GIGA, 'Q');
    format_volume_ntfs('Q');

    part_disk(&disk, 1 * MEGA, 'J');
    write_to_file(w!("\\\\.\\J:"), include_bytes!("uefi-ntfs.img").to_vec());
}

#[derive(Debug, Deserialize)]
struct Disk {
    #[serde(rename = "Number")]
    id: u32,
    #[serde(rename = "FriendlyName")]
    name: String,
}

fn ps_op2<T>(cmd: impl AsRef<OsStr>, op2: impl FnOnce(&[u8]) -> T) -> T {
    let output = Command::new("powershell.exe")
        .arg("-Command")
        .arg(cmd)
        .output()
        .unwrap();
    assert!(output.status.success());
    op2(&output.stdout)
}

fn ps_op(cmd: impl AsRef<OsStr>) {
    ps_op2(cmd, |_| ())
}

fn get_disk() -> Vec<Disk> {
    ps_op2(
        "ConvertTo-Json @(Get-Disk | Where-Object -Property BusType -eq 'USB' | Select-Object -Property Number,FriendlyName)",
        |out| serde_json::from_slice(out).unwrap(),
    )
}

fn clear_disk(disk: &Disk) {
    ps_op(format!(
        "Clear-Disk -Number {} -RemoveData -RemoveOEM -confirm:$false",
        disk.id
    ))
}

fn init_disk(disk: &Disk) {
    ps_op(format!(
        "Initialize-Disk -Number {} -PartitionStyle GPT",
        disk.id
    ))
}

fn part_disk(disk: &Disk, size: usize, l: char) {
    ps_op(format!(
        "New-Partition -DiskNumber {} -Size {} -DriveLetter {}",
        disk.id, size, l
    ))
}

fn format_volume_ntfs(l: char) {
    ps_op(format!("Format-Volume -DriveLetter {} -FileSystem NTFS", l))
}

fn write_to_file(path: PCWSTR, buffer: Vec<u8>) {
    unsafe {
        let file = CreateFileW(
            path,
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING,
            None,
        )
        .unwrap();
        let mut written = 0;
        WriteFile(file, Some(&buffer), Some(&mut written), None).unwrap();
        assert_eq!(written as usize, buffer.len());
        CloseHandle(file).unwrap();
    }
}
