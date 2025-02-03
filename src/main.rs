use std::fs;
use std::path::Path;
use std::process::Command;
use std::io::{self, Write, stdin};
use std::thread;
use std::time::Duration;
use std::env;

const BANNER: &str = r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║   ██╗  ██╗ ██████╗ ██████╗  █████╗ ██╗  ██╗███████╗███████╗ ██████╗       ║
║   ██║ ██╔╝██╔═══██╗██╔══██╗██╔══██╗██║ ██╔╝██╔════╝██╔════╝██╔════╝       ║
║   █████╔╝ ██║   ██║██║  ██║███████║█████╔╝ ███████╗█████╗  ██║            ║
║   ██╔═██╗ ██║   ██║██║  ██║██╔══██║██╔═██╗ ╚════██║██╔══╝  ██║            ║
║   ██║  ██╗╚██████╔╝██████╔╝██║  ██║██║  ██╗███████║███████╗╚██████╗       ║
║   ╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝       ║
║                                                                           ║
║                     Advanced Windows Log Cleaner                          ║
║                        By KodakSec 2025                                   ║
╚═══════════════════════════════════════════════════════════════════════════╝
"#;

const MENU: &str = r#"
╔═══════════════════════ SELECT FOLDERS TO CLEAN ══════════════════════════╗
║                                                                          ║
║  [1]  Windows Temp Files         [11] Discord Cache                      ║
║  [2]  Windows Prefetch           [12] Spotify Cache                      ║
║  [3]  Windows Update Cache       [13] Teams Cache & Logs                 ║
║  [4]  Windows Event Logs         [14] Visual Studio Logs                 ║
║  [5]  User Temp Files            [15] Windows Store Cache                ║
║  [6]  Chrome Cache               [16] Windows Defender Logs              ║
║  [7]  Firefox Cache              [17] System Error Memory Dump           ║
║  [8]  Edge Cache                 [18] Windows Debug Logs                 ║
║  [9]  Steam Logs                 [19] DirectX Shader Cache               ║
║  [10] Epic Games Logs            [20] Windows Error Reports              ║
║                                                                          ║
║  [21] Network Cache Reset        [22] DNS Cache Reset                    ║
║                                                                          ║
║  Type numbers to select (e.g.: 1 4 7 9) or type 'all' for everything     ║
╚══════════════════════════════════════════════════════════════════════════╝
"#;

struct CleanTarget<'a> {
    name: &'a str,
    path: &'a str,
}

fn get_available_drives() -> Vec<String> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        if Path::new(&drive).exists() {
            if let Ok(output) = Command::new("wmic")
                .args(["logicaldisk", "where", &format!("DeviceID='{}':", letter as char), "get", "DriveType"])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                if output.contains("3") {
                    drives.push(drive);
                }
            }
        }
    }
    drives
}

fn get_directory_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    total += metadata.len();
                }
            } else if path.is_dir() {
                total += get_directory_size(&path);
            }
        }
    }
    total
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

fn main() -> io::Result<()> {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", BANNER);
    thread::sleep(Duration::from_secs(1));

    let drives = get_available_drives();
    let mut all_targets = vec![];

    let mut targets = vec![
        CleanTarget { name: "Windows Temp", path: r"%TEMP%" },
        CleanTarget { name: "TMP", path: r"%TMP%" },
        CleanTarget { name: "Local Temp", path: r"%USERPROFILE%\AppData\Local\Temp" },
        CleanTarget { name: "Internet Files", path: r"%USERPROFILE%\AppData\Local\Microsoft\Windows\Temporary Internet Files" },
        CleanTarget { name: "Windows Logs", path: r"%WINDIR%\Logs" },
        CleanTarget { name: "System LogFiles", path: r"%WINDIR%\System32\LogFiles" },
        CleanTarget { name: "Event Logs", path: r"%WINDIR%\System32\Winevt\Logs" },
        CleanTarget { name: "Edge Default", path: r"%USERPROFILE%\AppData\Local\Microsoft\Edge\User Data\Default" },
        CleanTarget { name: "Edge Cache", path: r"%USERPROFILE%\AppData\Local\Microsoft\Edge\User Data\Default\Cache" },
        CleanTarget { name: "Edge Cookies", path: r"%USERPROFILE%\AppData\Local\Microsoft\Edge\User Data\Default\Cookies" },
        CleanTarget { name: "Edge Storage", path: r"%USERPROFILE%\AppData\Local\Microsoft\Edge\User Data\Default\Local Storage" },
        CleanTarget { name: "Edge Media", path: r"%USERPROFILE%\AppData\Local\Microsoft\Edge\User Data\Default\Media Cache" },
        CleanTarget { name: "INetCache", path: r"%USERPROFILE%\AppData\Local\Microsoft\Windows\INetCache" },
        CleanTarget { name: "INetCache IE", path: r"%USERPROFILE%\AppData\Local\Microsoft\Windows\INetCache\IE" },
        CleanTarget { name: "INetCache Low", path: r"%USERPROFILE%\AppData\Local\Microsoft\Windows\INetCache\Low" },
        CleanTarget { name: "Prefetch", path: r"%WINDIR%\Prefetch" },
        CleanTarget { name: "Printers Spool", path: r"%WINDIR%\System32\Spool\Printers" },
        CleanTarget { name: "Drivers Spool", path: r"%WINDIR%\System32\Spool\Drivers" },
        CleanTarget { name: "Admin Temp", path: r"C:\Users\Administrator\AppData\Local\Temp" },
        CleanTarget { name: "Firefox Cache", path: r"%USERPROFILE%\AppData\Roaming\Mozilla\Firefox\Profiles\*.default-release\cache2" },
        CleanTarget { name: "Firefox Cookies", path: r"%USERPROFILE%\AppData\Roaming\Mozilla\Firefox\Profiles\*.default-release\cookies.sqlite" },
        CleanTarget { name: "Firefox History", path: r"%USERPROFILE%\AppData\Roaming\Mozilla\Firefox\Profiles\*.default-release\formhistory.sqlite" },
        CleanTarget { name: "Firefox Sessions", path: r"%USERPROFILE%\AppData\Roaming\Mozilla\Firefox\Profiles\*.default-release\sessionstore-backups" },
        CleanTarget { name: "LibreWolf Cache", path: r"%USERPROFILE%\AppData\Roaming\LibreWolf\Profiles\*.default-release\cache2" },
        CleanTarget { name: "LibreWolf Cookies", path: r"%USERPROFILE%\AppData\Roaming\LibreWolf\Profiles\*.default-release\cookies.sqlite" },
        CleanTarget { name: "LibreWolf History", path: r"%USERPROFILE%\AppData\Roaming\LibreWolf\Profiles\*.default-release\formhistory.sqlite" },
        CleanTarget { name: "LibreWolf Sessions", path: r"%USERPROFILE%\AppData\Roaming\LibreWolf\Profiles\*.default-release\sessionstore-backups" },
        CleanTarget { name: "Chrome Cache", path: r"%USERPROFILE%\AppData\Local\Google\Chrome\User Data\Default\Cache" },
        CleanTarget { name: "Chrome Cookies", path: r"%USERPROFILE%\AppData\Local\Google\Chrome\User Data\Default\Cookies" },
        CleanTarget { name: "Chrome History", path: r"%USERPROFILE%\AppData\Local\Google\Chrome\User Data\Default\History" },
        CleanTarget { name: "Brave Cache", path: r"%USERPROFILE%\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\Cache" },
        CleanTarget { name: "Brave Cookies", path: r"%USERPROFILE%\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\Cookies" },
        CleanTarget { name: "Brave History", path: r"%USERPROFILE%\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\History" },
        CleanTarget { name: "Chrome Beta", path: r"%userprofile%\AppData\Local\Google\Chrome Beta\User Data\Default\Cache" },
        CleanTarget { name: "Chrome Canary", path: r"%userprofile%\AppData\Local\Google\Chrome SxS\User Data\Default\Cache" },
        CleanTarget { name: "Chromium", path: r"%userprofile%\AppData\Local\Chromium\User Data\Default\Cache" },
        CleanTarget { name: "Firefox Cache", path: r"%userprofile%\AppData\Local\Mozilla\Firefox\Profiles" },
        CleanTarget { name: "Firefox Beta", path: r"%userprofile%\AppData\Local\Mozilla\Firefox Beta\Profiles" },
        CleanTarget { name: "Firefox Dev", path: r"%userprofile%\AppData\Local\Mozilla\Firefox Developer Edition\Profiles" },
        CleanTarget { name: "Firefox Nightly", path: r"%userprofile%\AppData\Local\Mozilla\Firefox Nightly\Profiles" },
        CleanTarget { name: "Edge Beta", path: r"%userprofile%\AppData\Local\Microsoft\Edge Beta\User Data\Default\Cache" },
        CleanTarget { name: "Edge Dev", path: r"%userprofile%\AppData\Local\Microsoft\Edge Dev\User Data\Default\Cache" },
        CleanTarget { name: "Edge Canary", path: r"%userprofile%\AppData\Local\Microsoft\Edge SxS\User Data\Default\Cache" },
        CleanTarget { name: "Opera", path: r"%userprofile%\AppData\Local\Opera Software\Opera Stable\Cache" },
        CleanTarget { name: "Opera GX", path: r"%userprofile%\AppData\Local\Opera Software\Opera GX Stable\Cache" },
        CleanTarget { name: "Opera Beta", path: r"%userprofile%\AppData\Local\Opera Software\Opera Beta\Cache" },
        CleanTarget { name: "Opera Dev", path: r"%userprofile%\AppData\Local\Opera Software\Opera Developer\Cache" },
        CleanTarget { name: "Brave", path: r"%userprofile%\AppData\Local\BraveSoftware\Brave-Browser\User Data\Default\Cache" },
        CleanTarget { name: "Brave Beta", path: r"%userprofile%\AppData\Local\BraveSoftware\Brave-Browser-Beta\User Data\Default\Cache" },
        CleanTarget { name: "Vivaldi", path: r"%userprofile%\AppData\Local\Vivaldi\User Data\Default\Cache" },
        CleanTarget { name: "Waterfox", path: r"%userprofile%\AppData\Local\Waterfox\Profiles" },
        CleanTarget { name: "Pale Moon", path: r"%userprofile%\AppData\Local\Moonchild Productions\Pale Moon\Profiles" },
        CleanTarget { name: "Maxthon", path: r"%userprofile%\AppData\Local\Maxthon\Application\Cache" },
        CleanTarget { name: "Torch", path: r"%userprofile%\AppData\Local\Torch\User Data\Default\Cache" },
        CleanTarget { name: "Comodo Dragon", path: r"%userprofile%\AppData\Local\Comodo\Dragon\User Data\Default\Cache" },
        CleanTarget { name: "Yandex", path: r"%userprofile%\AppData\Local\Yandex\YandexBrowser\User Data\Default\Cache" },
        CleanTarget { name: "UC Browser", path: r"%userprofile%\AppData\Local\UCBrowser\User Data\Default\Cache" },
        CleanTarget { name: "Cent Browser", path: r"%userprofile%\AppData\Local\CentBrowser\User Data\Default\Cache" },
        CleanTarget { name: "Slimjet", path: r"%userprofile%\AppData\Local\Slimjet\User Data\Default\Cache" },
        CleanTarget { name: "Safari Cache", path: r"%userprofile%\AppData\Local\Apple Computer\Safari\Cache" },
        CleanTarget { name: "SeaMonkey", path: r"%userprofile%\AppData\Local\Mozilla\SeaMonkey\Profiles" },
        CleanTarget { name: "Chrome Session", path: r"%userprofile%\AppData\Local\Google\Chrome\User Data\Default\Sessions" },
        CleanTarget { name: "Edge Session", path: r"%userprofile%\AppData\Local\Microsoft\Edge\User Data\Default\Sessions" },
        CleanTarget { name: "Firefox Cache2", path: r"%userprofile%\AppData\Local\Mozilla\Firefox\Profiles\*.default\cache2" },
        CleanTarget { name: "Edge Cookies", path: r"%userprofile%\AppData\Local\Microsoft\Edge\User Data\Default\Cookies" },
        CleanTarget { name: "Edge Session", path: r"%userprofile%\AppData\Local\Microsoft\Edge\User Data\Default\Sessions" },
        CleanTarget { name: "User Temp", path: r"%temp%" },
        CleanTarget { name: "Steam Logs", path: r"%programfiles(x86)%\Steam\logs" },
        CleanTarget { name: "Epic Games", path: r"%programdata%\Epic\EpicGamesLauncher\Data\Logs" },
        CleanTarget { name: "Discord", path: r"%localappdata%\Discord\Cache" },
        CleanTarget { name: "Spotify", path: r"%appdata%\Spotify\Users" },
        CleanTarget { name: "Teams", path: r"%localappdata%\Microsoft\Teams" },
        CleanTarget { name: "Visual Studio", path: r"%localappdata%\Microsoft\VisualStudio\Logs" },
        CleanTarget { name: "Windows Store", path: r"%localappdata%\Packages" },
        CleanTarget { name: "DirectX Shaders", path: r"%localappdata%\D3DSCache" },
        CleanTarget { name: "Windows Update", path: r"C:\Windows\SoftwareDistribution\Download" },
        CleanTarget { name: "Windows Defender", path: r"C:\ProgramData\Microsoft\Windows Defender\Scans\History" },
        CleanTarget { name: "Memory Dumps", path: r"C:\Windows\Minidump" },
        CleanTarget { name: "Debug Logs", path: r"C:\Windows\debug" },
        CleanTarget { name: "Error Reports", path: r"C:\ProgramData\Microsoft\Windows\WER" },
        CleanTarget { name: "Tor Browser", path: r"%userprofile%\AppData\Local\TorBrowser-Data\Browser\*\Data\Browser\Cache" },
        CleanTarget { name: "Tor Browser Cookies", path: r"%userprofile%\AppData\Local\TorBrowser-Data\Browser\*\Data\Browser\profile.default\cookies.sqlite" },
        CleanTarget { name: "Tor Browser History", path: r"%userprofile%\AppData\Local\TorBrowser-Data\Browser\*\Data\Browser\profile.default\places.sqlite" },
        CleanTarget { name: "Steam", path: r"%programfiles(x86)%\Steam\logs" },
        CleanTarget { name: "Steam Download", path: r"%programfiles(x86)%\Steam\steamapps\downloading" },
        CleanTarget { name: "Steam Temp", path: r"%programfiles(x86)%\Steam\depotcache" },
        CleanTarget { name: "Steam Crash", path: r"%programfiles(x86)%\Steam\dumps" },
        CleanTarget { name: "Epic Games Cache", path: r"%localappdata%\EpicGamesLauncher\Saved\Cache" },
        CleanTarget { name: "Epic Games WebCache", path: r"%localappdata%\EpicGamesLauncher\Saved\webcache" },
        CleanTarget { name: "Origin", path: r"%programdata%\Origin\Logs" },
        CleanTarget { name: "Origin Cache", path: r"%localappdata%\Origin\Cache" },
        CleanTarget { name: "Origin ThinSetup", path: r"%localappdata%\Origin\ThinSetup" },
        CleanTarget { name: "Ubisoft Connect", path: r"%localappdata%\Ubisoft Game Launcher\logs" },
        CleanTarget { name: "Ubisoft Cache", path: r"%localappdata%\Ubisoft Game Launcher\cache" },
        CleanTarget { name: "Ubisoft WebCache", path: r"%localappdata%\Ubisoft Game Launcher\webcache" },
        CleanTarget { name: "Battle.net", path: r"%programdata%\Battle.net\Setup\Logs" },
        CleanTarget { name: "Battle.net Cache", path: r"%programdata%\Battle.net\Agent\data" },
        CleanTarget { name: "Battle.net Temp", path: r"%localappdata%\Blizzard Entertainment\Battle.net\Cache" },
        CleanTarget { name: "GOG Galaxy", path: r"%programdata%\GOG.com\Galaxy\logs" },
        CleanTarget { name: "GOG Galaxy Cache", path: r"%localappdata%\GOG.com\Galaxy\webcache" },
        CleanTarget { name: "GOG Galaxy Storage", path: r"%localappdata%\GOG.com\Galaxy\Storage" },
        CleanTarget { name: "Riot Games", path: r"%localappdata%\Riot Games\Logs" },
        CleanTarget { name: "Riot Client", path: r"%localappdata%\Riot Games\Riot Client\Data" },
        CleanTarget { name: "Riot Cache", path: r"%localappdata%\Riot Games\Riot Client\Cache" },
        CleanTarget { name: "Xbox", path: r"%localappdata%\Packages\Microsoft.GamingApp_8wekyb3d8bbwe\LocalCache" },
        CleanTarget { name: "Xbox Logs", path: r"%localappdata%\Packages\Microsoft.XboxGamingOverlay_8wekyb3d8bbwe\LocalState\DiagOutputDir" },       
        CleanTarget { name: "Bethesda Launcher", path: r"%localappdata%\Bethesda.net Launcher\logs" },
        CleanTarget { name: "Amazon Games", path: r"%localappdata%\Amazon Games\Logs" },
        CleanTarget { name: "itch.io", path: r"%appdata%\itch\logs" },
        CleanTarget { name: "Rockstar Games", path: r"%localappdata%\Rockstar Games\Launcher\Log" },       
        CleanTarget { name: "CCleaner", path: r"%programdata%\CCleaner\Logs" },
        CleanTarget { name: "MSI Afterburner", path: r"%programfiles(x86)%\MSI Afterburner\Log" },
        CleanTarget { name: "GeForce Experience", path: r"%programdata%\NVIDIA Corporation\GeForce Experience\Logs" },
        CleanTarget { name: "AMD Radeon", path: r"%programdata%\AMD\RadeonSettings\Logs" },
        CleanTarget { name: "Corsair iCUE", path: r"%programdata%\Corsair\CUE\logs" },
        CleanTarget { name: "Razer Synapse", path: r"%programdata%\Razer\Synapse\Logs" },
        CleanTarget { name: "LogiTech G HUB", path: r"%programdata%\LGHUB\logs" },        
        CleanTarget { name: "Windows CBS Logs", path: r"C:\Windows\Logs\CBS" },
        CleanTarget { name: "Windows Setup Logs", path: r"C:\Windows\Panther" },
        CleanTarget { name: "Windows Update Logs", path: r"C:\Windows\Logs\WindowsUpdate" },
        CleanTarget { name: "Windows Security Logs", path: r"C:\Windows\System32\LogFiles\Security" },
        CleanTarget { name: "Windows NetSetup", path: r"C:\Windows\Debug\NetSetup" },
        CleanTarget { name: "Windows Performance", path: r"C:\Windows\Performance\WinSAT" },
        CleanTarget { name: "System Recycle Bin", path: r"C:\$Recycle.Bin" },
        CleanTarget { name: "Windows Font Cache", path: r"C:\Windows\ServiceProfiles\LocalService\AppData\Local\FontCache" },
        CleanTarget { name: "Windows Installer Logs", path: r"C:\Windows\Installer\*.log" },
        CleanTarget { name: "Windows Error Logs", path: r"C:\Windows\System32\LogFiles\WMI" },
        CleanTarget { name: "Windows Search Data", path: r"C:\ProgramData\Microsoft\Search\Data\Applications\Windows" },
        CleanTarget { name: "Unity Cache", path: r"%userprofile%\AppData\Local\Unity\Cache" },
        CleanTarget { name: "Unreal Engine Logs", path: r"%userprofile%\AppData\Local\UnrealEngine\*\Saved\Logs" },
        CleanTarget { name: "EA Desktop", path: r"%programdata%\EA Desktop\Logs" },
    ];

    all_targets.append(&mut targets);
    
    // Ajouter les Recycle Bin après
    for drive in &drives {
        all_targets.push(CleanTarget {
            name: Box::leak(format!("Recycle Bin ({})", drive).into_boxed_str()),
            path: Box::leak(format!("{}$Recycle.Bin", drive).into_boxed_str()),
        });
    }

    println!("\n💽 Drives detected: {}", drives.join(", "));
    
    thread::sleep(Duration::from_secs(2));
    println!("\n{}", MENU);
    print!("Enter your choice: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selected_indices: Vec<usize> = if input.trim().to_lowercase() == "all" {
        (0..all_targets.len()).collect()
    } else {
        input
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .filter(|&n| n > 0 && n <= all_targets.len())
            .map(|n| n - 1)
            .collect()
    };

    if selected_indices.is_empty() {
        println!("❌ No valid selections made!");
        return Ok(());
    }

    let total = selected_indices.len();
    println!("\n🧹 Starting cleanup process...");

    for (idx, &target_idx) in selected_indices.iter().enumerate() {
        show_progress_bar(idx + 1, total);
        clean_directory(all_targets[target_idx].path)?;
    }

    if input == "all" || input.contains("21") || input.contains("22") {
        println!("\n🔄 Resetting network settings...");
        let commands = vec![
            ("ipconfig", vec!["/flushdns"]),
            ("netsh", vec!["winsock", "reset"]),
            ("netsh", vec!["int", "ip", "reset"]),
        ];

        for (cmd, args) in commands {
            execute_command(cmd, &args)?;
        }
    }

    println!("\n╔════════════════════════════════════════════════╗");
    println!("║             Cleanup Complete! ✨               ║");
    println!("╚════════════════════════════════════════════════╝");
    Ok(())
}

fn expand_path(path: &str) -> String {
    if path.contains('%') {
        path.replace("%userprofile%", &std::env::var("USERPROFILE").unwrap_or_default())
            .replace("%localappdata%", &std::env::var("LOCALAPPDATA").unwrap_or_default())
            .replace("%appdata%", &std::env::var("APPDATA").unwrap_or_default())
            .replace("%temp%", &std::env::var("TEMP").unwrap_or_default())
            .replace("%programdata%", &std::env::var("PROGRAMDATA").unwrap_or_default())
            .replace("%programfiles(x86)%", &std::env::var("PROGRAMFILES(X86)").unwrap_or_default())
    } else {
        path.to_string()
    }
}

fn show_progress_bar(current: usize, total: usize) {
    let percentage = (current as f32 / total as f32 * 100.0) as usize;
    let filled = (percentage / 2) as usize;
    let empty = 50 - filled;
    
    print!("\r[");
    for _ in 0..filled {
        print!("█");
    }
    for _ in 0..empty {
        print!("░");
    }
    print!("] {}% ", percentage);
    io::stdout().flush().unwrap();
}

fn clean_directory(path: &str) -> io::Result<()> {
    let expanded_path = expand_path(path);
    let path = Path::new(&expanded_path);
    if !path.exists() {
        return Ok(());
    }

    println!("🗑️ Cleaning: {}", path.display());

    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            let _ = if path.is_file() {
                fs::remove_file(path)
            } else if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                Ok(())
            };
        }
    }
    Ok(())
}

fn execute_command(command: &str, args: &[&str]) -> io::Result<()> {
    println!("🔧 Exécution de: {} {}", command, args.join(" "));
    Command::new(command)
        .args(args)
        .output()?;
    Ok(())
}
