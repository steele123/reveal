<#
.SYNOPSIS
  Posts a Reveal release to the Discord #releases channel.

.DESCRIPTION
  Reads the GitHub release notes for the version (default: the version in tauri.conf.json)
  and posts them as an embed through the channel's webhook. Run it after the GitHub release
  is published. Use -Ping to mention the Release Pings role.

  The webhook URL is read from the REVEAL_DISCORD_RELEASES_WEBHOOK environment variable.
  Treat it like a password: anyone with it can post to the channel.

.EXAMPLE
  .\scripts\announce-release.ps1 -Ping
#>
[CmdletBinding()]
param(
    [string]$Version,
    [switch]$Ping,
    [string]$WebhookUrl = [Environment]::GetEnvironmentVariable("REVEAL_DISCORD_RELEASES_WEBHOOK", "User"),
    [string]$PingRoleId = "1562269807246843915"
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
if (-not $Version) {
    $Version = (Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json).package.version
}
if (-not $WebhookUrl) { throw "Set REVEAL_DISCORD_RELEASES_WEBHOOK to the #releases webhook URL." }

$previous = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$json = gh release view "v$Version" -R steele123/reveal --json name,body,url,publishedAt 2>&1
$ErrorActionPreference = $previous
if ($LASTEXITCODE -ne 0) { throw "GitHub release v$Version not found: $json" }
$release = $json | ConvertFrom-Json

# Embeds don't render "## Heading" lines, so turn them into bold lines.
$notes = ($release.body -split "`r?`n" | ForEach-Object { $_ -replace '^#{1,6}\s+(.+)$', '**$1**' }) -join "`n"
$notes = $notes.Trim()
$download = [string][char]0x2B07 + [char]0xFE0F
$dot = [char]0x00B7
$footerLinks = "`n`n$download [Download](https://lobbyreveal.app/api/reveal/download) $dot [Release notes]($($release.url))`nAlready have Reveal? It updates itself next time you open it."
$limit = 4096 - $footerLinks.Length
if ($notes.Length -gt $limit) { $notes = $notes.Substring(0, $limit - 1) + [char]0x2026 }

$payload = @{
    username   = "Reveal Releases"
    avatar_url = "https://raw.githubusercontent.com/steele123/reveal/main/branding/discord/server-icon-512.png"
    embeds     = @(@{
        title       = "Reveal v$Version is out"
        url         = $release.url
        description = $notes + $footerLinks
        color       = 0x3B82F6
        timestamp   = $release.publishedAt
        footer      = @{ text = "lobbyreveal.app" }
    })
    allowed_mentions = @{ parse = @(); roles = @() }
}
if ($Ping) {
    $payload.content = "<@&$PingRoleId>"
    $payload.allowed_mentions.roles = @($PingRoleId)
}

# Send as UTF-8 bytes so emoji and accents survive Windows PowerShell.
$body = [Text.Encoding]::UTF8.GetBytes(($payload | ConvertTo-Json -Depth 6))
$message = Invoke-RestMethod -Method Post -Uri "$WebhookUrl`?wait=true" -ContentType "application/json; charset=utf-8" -Body $body
Write-Host "Posted Reveal v$Version to #releases (message $($message.id))$(if ($Ping) { ' with a Release Pings mention' })"
