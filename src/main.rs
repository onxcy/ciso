use std::{fs::DirEntry, io::Write, path::Path};

use powershell::PowerShell;
use serde::Deserialize;

fn main() {
    let iso_path = std::env::args().skip(1).next().unwrap();
    let iso_path2 = iso_path.clone();

    std::panic::set_hook(Box::new(move |info| {
        PowerShell::new()
            .no_confirm()
            .add_variable("iso_path", &iso_path2)
            .invoke(include_str!("y.ps1"));
        eprintln!("{}", info.to_string());
        std::process::exit(1);
    }));

    let out = PowerShell::new()
        .no_confirm()
        .add_variable("iso_path", &iso_path)
        .invoke(include_str!("x.ps1"));
    let data: OutData = serde_json::from_slice(&out.stdout).unwrap();

    let uefi_path = data.uefi.access_paths[0].as_str();
    let uefi_path = &uefi_path[..uefi_path.len() - 1];
    std::fs::OpenOptions::new()
        .write(true)
        .open(uefi_path)
        .unwrap()
        .write_all(include_bytes!("uefi-ntfs.img"))
        .unwrap();

    let piso = Path::new(&data.iso.path);
    let pntfs = Path::new(&data.ntfs.path);

    visit_dirs(
        piso,
        &|entry| {
            let p1 = entry.path();
            let p2 = pntfs.join(p1.strip_prefix(piso).unwrap());
            std::fs::copy(p1, p2).unwrap();
        },
        &|entry| {
            let p1 = entry.path();
            let p2 = pntfs.join(p1.strip_prefix(piso).unwrap());
            std::fs::create_dir_all(p2).unwrap();
        },
    )
    .unwrap();

    PowerShell::new()
        .no_confirm()
        .add_variable("iso_path", &iso_path)
        .invoke(include_str!("y.ps1"));
}

#[derive(Debug, Deserialize)]
struct OutData {
    ntfs: Volume,
    iso: Volume,
    uefi: Partition,
}

#[derive(Debug, Deserialize)]
struct Volume {
    #[serde(rename = "Path")]
    path: String,
}

#[derive(Debug, Deserialize)]
struct Partition {
    #[serde(rename = "AccessPaths")]
    access_paths: [String; 1],
}

fn visit_dirs(
    dir: &Path,
    visit_file: &impl Fn(&DirEntry),
    visit_dir: &impl Fn(&DirEntry),
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&entry);
            visit_dirs(&path, visit_file, visit_dir)?;
        } else {
            visit_file(&entry);
        }
    }
    Ok(())
}
