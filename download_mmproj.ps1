$url = "https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/mmproj-qwen2.5-7b-instruct-f16.gguf"
$out = "D:\llm\models\mmproj-qwen.gguf"
Write-Output "Downloading mmproj from $url ..."
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
$wc = New-Object System.Net.WebClient
$wc.DownloadFile($url, $out)
Write-Output "Saved to $out"
$size = (Get-Item $out).Length
Write-Output "Size: $($size / 1MB) MB"
