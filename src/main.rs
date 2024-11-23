use core::str;
use std::process::Command;

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
    part_disk(&disk, 8 * GIGA, 'X');
    part_disk(&disk, 1 * MEGA, 'Y');
    format_volume_ntfs('X');
    write_to_file(w!("\\\\.\\Y:"), include_bytes!("uefi-ntfs.img").to_vec());
}

#[derive(Debug, Deserialize)]
struct Disk {
    #[serde(rename = "Number")]
    id: u32,
    #[serde(rename = "FriendlyName")]
    name: String,
}

fn ps_op<T>(op1: impl FnOnce(&mut Command) -> &mut Command, op2: impl FnOnce(&[u8]) -> T) -> T {
    let output = op1(Command::new("powershell.exe").arg("-Command"))
        .output()
        .unwrap();
    assert!(output.status.success());
    op2(&output.stdout)
}

fn get_disk() -> Vec<Disk> {
    ps_op(
        |cmd| {
            cmd.arg("ConvertTo-Json @(Get-Disk | Where-Object -Property BusType -eq 'USB' | Select-Object -Property Number,FriendlyName)")
        },
        |out| serde_json::from_slice(out).unwrap(),
    )
}

fn clear_disk(disk: &Disk) {
    ps_op(
        |cmd| {
            cmd.arg(format!(
                "Clear-Disk -Number {} -RemoveData -RemoveOEM -confirm:$false",
                disk.id
            ))
        },
        |_| (),
    )
}

fn init_disk(disk: &Disk) {
    ps_op(
        |cmd| {
            cmd.arg(format!(
                "Initialize-Disk -Number {} -PartitionStyle GPT",
                disk.id
            ))
        },
        |_| (),
    )
}

fn part_disk(disk: &Disk, size: usize, l: char) {
    ps_op(
        |cmd| {
            cmd.arg(format!(
                "New-Partition -DiskNumber {} -Size {} -DriveLetter {}",
                disk.id, size, l
            ))
        },
        |_| (),
    )
}

fn format_volume_ntfs(l: char) {
    ps_op(
        |cmd| cmd.arg(format!("Format-Volume -DriveLetter {} -FileSystem NTFS", l)),
        |_| (),
    )
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
