use std::{
    alloc::Layout,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    process::{Command, Stdio},
};

use hex_literal::hex;
use sha2::{Digest, Sha256};

const PS_NAME: &str = "powershell";
const PS_USE_STDIN: &[&str] = &["-Command", "-"];
const PS_SCRIPT: &[u8] = concat!(include_str!("x.ps1"), '\n').as_bytes();

const GENERIC_WRITE: u32 = 0x40000000;
const GENERIC_READ: u32 = 0x80000000;

const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;

const UEFI_NTFS_IMG: &[u8; 1024 * 1024] = include_bytes!("uefi-ntfs.img");
const UEFI_NTFS_SHA256: &[u8; 32] =
    &hex!("25D6BB709B8C952799C0AF3DE29356B33AA64FCBD9A98B3625AC0E806EE49C7B");

const PAGE_ALIGN: usize = 4096;

fn main() {
    let out = run_pscmd();
    let out = out.trim();
    let uefi_path = &out[..out.len() - 1];

    img2disk(uefi_path);
    checkimg(uefi_path);
}

fn run_pscmd() -> String {
    let mut ps = Command::new(PS_NAME)
        .args(PS_USE_STDIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    ps.stdin.as_mut().unwrap().write_all(PS_SCRIPT).unwrap();

    assert!(ps.wait().unwrap().success());

    let mut out = String::new();
    ps.stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    out
}

fn img2disk(path: impl AsRef<Path>) {
    let buffer = alloc_buffer();
    buffer.copy_from_slice(UEFI_NTFS_IMG);

    let mut file = open_file(path);
    let n = file.write(buffer).unwrap();

    assert_eq!(n, UEFI_NTFS_IMG.len());
}

fn checkimg(path: impl AsRef<Path>) {
    let buffer = alloc_buffer();

    let mut file = open_file(path);
    let n = file.read(buffer).unwrap();

    assert_eq!(n, UEFI_NTFS_IMG.len());

    let mut hasher = Sha256::new();
    hasher.update(buffer);
    assert_eq!(&hasher.finalize()[..], UEFI_NTFS_SHA256);
}

fn alloc_buffer() -> &'static mut [u8] {
    unsafe {
        let ptr = std::alloc::alloc_zeroed(
            Layout::from_size_align(UEFI_NTFS_IMG.len(), PAGE_ALIGN).unwrap(),
        );
        assert!(!ptr.is_null());
        std::slice::from_raw_parts_mut(ptr, UEFI_NTFS_IMG.len())
    }
}

fn open_file(path: impl AsRef<Path>) -> File {
    OpenOptions::new()
        .access_mode(GENERIC_READ | GENERIC_WRITE)
        .share_mode(0)
        .custom_flags(FILE_FLAG_WRITE_THROUGH | FILE_FLAG_NO_BUFFERING)
        .open(path)
        .unwrap()
}
