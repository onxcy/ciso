use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::windows::fs::OpenOptionsExt,
    process::{Command, Stdio},
};

const PS_NAME: &str = "powershell";
const PS_USE_STDIN: &[&str] = &["-Command", "-"];
const PS_SCRIPT: &[u8] = concat!(include_str!("x.ps1"), '\n').as_bytes();
const GENERIC_WRITE: u32 = 0x40000000;
const UEFI_NTFS_IMG: &[u8] = include_bytes!("uefi-ntfs.img");

fn main() {
    let mut ps = Command::new(PS_NAME)
        .args(PS_USE_STDIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
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
    let out = out.trim();
    let uefi_path = &out[..out.len() - 1];
    let mut file = OpenOptions::new()
        .access_mode(GENERIC_WRITE)
        .share_mode(0)
        .open(uefi_path)
        .unwrap();
    file.write_all(UEFI_NTFS_IMG).unwrap();
    file.flush().unwrap();
}
