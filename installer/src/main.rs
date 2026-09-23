use snag_core::quiet;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const PAYLOAD: &[u8] = include_bytes!(env!("PAYLOAD_EXE"));
const PRODUCT: &str = env!("PRODUCT_NAME");
const EXE_NAME: &str = env!("EXE_NAME");
const REG_KEY: &str = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

const VERSION: &str = match option_env!("APP_VERSION") {
    Some(value) => value,
    None => "0.0.0",
};

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).map(PathBuf::from)
}

fn install_dir() -> Option<PathBuf> {
    Some(env_path("LOCALAPPDATA")?.join("Programs").join(PRODUCT))
}

fn powershell(script: &str) -> bool {
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    quiet(&mut cmd);
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn reg(args: &[&str]) -> bool {
    let mut cmd = Command::new("reg");
    cmd.args(args);
    quiet(&mut cmd);
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn shortcuts(target: &Path, remove: bool) -> bool {
    let working = target
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let action = if remove {
        "if (Test-Path $p) { Remove-Item $p -Force }".to_string()
    } else {
        format!(
            "$s = $ws.CreateShortcut($p); $s.TargetPath = '{}'; $s.WorkingDirectory = '{}'; \
             $s.Description = '{}'; $s.Save()",
            target.display(),
            working,
            PRODUCT
        )
    };

    powershell(&format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $desktop = [Environment]::GetFolderPath('Desktop'); \
         $menu = [Environment]::GetFolderPath('Programs'); \
         foreach ($p in @(\"$desktop\\{name}.lnk\", \"$menu\\{name}.lnk\")) {{ {action} }}",
        name = PRODUCT,
        action = action
    ))
}

fn pause(message: &str) {
    println!("\n{message}");
    let _ = std::io::stdout().flush();
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer);
}

fn install() {
    println!("  Installing {PRODUCT}\n");

    let Some(dir) = install_dir() else {
        pause("Could not read LOCALAPPDATA. Press Enter to exit.");
        return;
    };

    if let Err(err) = std::fs::create_dir_all(&dir) {
        pause(&format!(
            "Could not create {}: {err}\nPress Enter to exit.",
            dir.display()
        ));
        return;
    }

    let target = dir.join(EXE_NAME);
    println!("  Copying to {}", target.display());

    if let Err(err) = std::fs::write(&target, PAYLOAD) {
        let hint = if err.raw_os_error() == Some(32) {
            "The app is running. Close it and run this installer again."
        } else {
            "Could not write the file."
        };
        pause(&format!("  ERROR: {err}\n  {hint}\n\nPress Enter to exit."));
        return;
    }

    let uninstaller = dir.join("Uninstall.exe");
    if let Ok(current) = std::env::current_exe() {
        let _ = std::fs::copy(&current, &uninstaller);
    }

    println!("  Creating Desktop and Start Menu shortcuts");
    if !shortcuts(&target, false) {
        println!("  (could not create the shortcuts, but the app is installed)");
    }

    println!("  Registering under Add or remove programs");
    let key = format!("{REG_KEY}\\{PRODUCT}");
    let size = (PAYLOAD.len() / 1024).to_string();
    let uninstall_cmd = format!("\"{}\" --uninstall", uninstaller.display());
    let entries: [(&str, &str, &str); 8] = [
        ("DisplayName", "REG_SZ", PRODUCT),
        ("DisplayVersion", "REG_SZ", VERSION),
        ("Publisher", "REG_SZ", "rysted"),
        ("DisplayIcon", "REG_SZ", &target.display().to_string()),
        ("InstallLocation", "REG_SZ", &dir.display().to_string()),
        ("UninstallString", "REG_SZ", &uninstall_cmd),
        ("EstimatedSize", "REG_DWORD", &size),
        ("NoModify", "REG_DWORD", "1"),
    ];
    for (name, kind, value) in entries {
        reg(&["add", &key, "/v", name, "/t", kind, "/d", value, "/f"]);
    }

    println!("\n  Done. {PRODUCT} is installed at:");
    println!("  {}", dir.display());
    println!("\n  The shortcut is on your Desktop.");

    print!("\n  Open it now? [Y/n] ");
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    let _ = std::io::stdin().read_line(&mut answer);
    if !answer.trim().eq_ignore_ascii_case("n") {
        let mut cmd = Command::new(&target);
        cmd.current_dir(&dir);
        let _ = cmd.spawn();
    }
}

fn settings_dir() -> Option<PathBuf> {
    Some(env_path("APPDATA")?.join("Snag"))
}

fn other_build_installed() -> bool {
    let Some(base) = env_path("LOCALAPPDATA").map(|path| path.join("Programs")) else {
        return false;
    };
    let Ok(entries) = std::fs::read_dir(base) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        name != PRODUCT && name.starts_with("Snag") && entry.path().is_dir()
    })
}

fn uninstall() {
    println!("  Uninstalling {PRODUCT}\n");

    let Some(dir) = install_dir() else { return };
    let target = dir.join(EXE_NAME);

    println!("  Removing shortcuts");
    shortcuts(&target, true);

    println!("  Removing the registry entry");
    reg(&["delete", &format!("{REG_KEY}\\{PRODUCT}"), "/f"]);

    if let Some(settings) = settings_dir() {
        if settings.exists() {
            if other_build_installed() {
                println!("  Keeping settings: the other Snag build still uses them");
            } else {
                println!("  Removing settings");
                let _ = std::fs::remove_dir_all(&settings);
            }
        }
    }

    println!("  Deleting {}", dir.display());

    let quoted = format!("\"{}\"", dir.display());
    let attempt =
        |seconds: u32| format!("ping -n {seconds} 127.0.0.1 >nul & rmdir /s /q {quoted} >nul 2>&1");
    let script = format!("{} & {} & {}", attempt(2), attempt(3), attempt(5));

    let mut cmd = Command::new("cmd");
    // Passed as a normal argument, the script would be wrapped in quotes and
    // the quotes around the path escaped as \", which cmd does not understand:
    // every product name with a space in it then failed to delete. Only a raw
    // command line reaches cmd unchanged.
    cmd.raw_arg(format!("/c {script}"));
    // cmd inherits our working directory, and when the uninstaller is launched
    // by double-clicking it that is the folder being deleted. Windows refuses
    // to remove a folder a process is sitting in.
    cmd.current_dir(std::env::temp_dir());
    quiet(&mut cmd);
    let _ = cmd.spawn();

    println!("\n  Done. Your downloads were left untouched.");
}

fn main() {
    println!("\n  ============================================");
    println!("  {PRODUCT}");
    println!("  ============================================\n");

    if std::env::args().any(|a| a == "--uninstall") {
        uninstall();
    } else {
        install();
    }
}
