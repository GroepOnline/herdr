# installed by herdr
# managed by herdr; reinstalling or updating the integration overwrites this file.
# HERDR_INTEGRATION_ID=freebuff
# HERDR_INTEGRATION_VERSION=1
#
# Freebuff is currently screen-detected: it has no documented hook lifecycle
# yet, so this script is staged for the day it does. Until freebuff invokes
# hooks, herdr keeps using screen detection as the state authority.

param([string]$Action = "")

if ($Action -ne "session") { exit 0 }
if ($env:HERDR_ENV -ne "1") { exit 0 }
if ([string]::IsNullOrWhiteSpace($env:HERDR_PANE_ID)) { exit 0 }

$inputText = [Console]::In.ReadToEnd()
try {
    $payload = if ([string]::IsNullOrWhiteSpace($inputText)) { $null } else { $inputText | ConvertFrom-Json }
} catch {
    $payload = $null
}

$event = if ($null -ne $payload -and $payload.hook_event_name -is [string]) {
    $payload.hook_event_name
} elseif ($null -ne $payload -and $payload.hookEventName -is [string]) {
    $payload.hookEventName
} else {
    $null
}
if ($null -ne $event -and $event -notin @("SessionStart", "session_start", "sessionStart")) { exit 0 }

$sessionId = $env:FREEBUFF_SESSION_ID
if ([string]::IsNullOrWhiteSpace($sessionId) -and $null -ne $payload) {
    if ($payload.session_id -is [string]) { $sessionId = $payload.session_id }
    elseif ($payload.sessionId -is [string]) { $sessionId = $payload.sessionId }
    elseif ($payload.conversation_id -is [string]) { $sessionId = $payload.conversation_id }
    elseif ($payload.conversationId -is [string]) { $sessionId = $payload.conversationId }
}
if ([string]::IsNullOrWhiteSpace($sessionId)) { exit 0 }

$seq = [DateTime]::UtcNow.Ticks
$herdr = if ([string]::IsNullOrWhiteSpace($env:HERDR_BIN_PATH)) { "herdr" } else { $env:HERDR_BIN_PATH }
try {
    & $herdr pane report-agent-session $env:HERDR_PANE_ID --source herdr:freebuff --agent freebuff --seq $seq --agent-session-id $sessionId 2>$null | Out-Null
} catch {
}
