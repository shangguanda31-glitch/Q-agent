use std::process::Command;

pub fn send_toast(title: &str, body: &str) {
    let safe_title = title.replace('\'', "''");
    let safe_body = body.replace('\'', "''");

    let ps_script = format!(
        r#"
$null = [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]
$template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
$textNodes = $template.GetElementsByTagName('text')
$textNodes.Item(0).AppendChild($template.CreateTextNode('{0}')) | Out-Null
$textNodes.Item(1).AppendChild($template.CreateTextNode('{1}')) | Out-Null
$toastEl = $template.SelectSingleNode('/toast')
$toastEl.SetAttribute('duration', 'long')
$toast = [Windows.UI.Notifications.ToastNotification]::new($template)
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
