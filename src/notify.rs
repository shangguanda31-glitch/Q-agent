use std::process::Command;

pub fn send_toast(title: &str, body: &str) {
    // XML-encode to prevent PowerShell injection
    let safe_title = title.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('\'', "&apos;").replace('"', "&quot;");
    let safe_body = body.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('\'', "&apos;").replace('"', "&quot;");

    let ps_script = format!(
        r#"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom, ContentType = WindowsRuntime] | Out-Null
$xml = [Windows.Data.Xml.Dom.XmlDocument]::new()
$xml.LoadXml(@"
<toast duration=""long"">
  <visual>
    <binding template=""ToastText02"">
      <text id=""1"">{0}</text>
      <text id=""2"">{1}</text>
    </binding>
  </visual>
</toast>
"@)
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('QAgent').Show($toast)
"#,
        safe_title, safe_body
    );

    match Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
    {
        Ok(_) => {}
        Err(e) => tracing::error!("Failed to send notification: {}", e),
    }
}

pub fn notify_high_priority(summary: &str, sender: &str, source: &str) {
    let title = format!("[高优先级] {}", summary);
    let body = format!("来自 {} · {}", sender, source);
    send_toast(&title, &body);
}

pub fn notify_schedule(title: &str, time: Option<&str>, description: Option<&str>) {
    let body = match (time, description) {
        (Some(t), Some(d)) => format!("时间: {} | 描述: {}", t, d),
        (Some(t), None) => format!("时间: {}", t),
        (None, Some(d)) => format!("描述: {}", d),
        (None, None) => "新日程待确认".to_string(),
    };
    send_toast(&format!("[日程] {}", title), &body);
}
