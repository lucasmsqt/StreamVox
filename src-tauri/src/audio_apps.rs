use std::process::Command;

pub fn list_audio_playing_apps() -> Vec<String> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Get-Process | Where-Object {$_.MainWindowTitle -ne \"\"} | Format-Table Name, Id, MainWindowTitle -AutoSize")
        .output()
        .expect("Failed to execute command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    output_str.split('\n').map(|s| s.to_string()).collect()
}
