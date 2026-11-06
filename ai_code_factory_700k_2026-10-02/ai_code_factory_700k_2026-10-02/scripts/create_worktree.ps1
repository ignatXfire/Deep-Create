param(
  [Parameter(Mandatory=$true)][string]$TaskId,
  [Parameter(Mandatory=$true)][string]$BaseSha,
  [string]$Repo = ".",
  [string]$OutRoot = "../agent-worktrees"
)
$ErrorActionPreference = "Stop"
$slug = $TaskId.ToLower()
$path = Join-Path $OutRoot $slug
$branch = "agent/$slug"
Push-Location $Repo
try {
  git rev-parse --verify $BaseSha | Out-Null
  if (Test-Path $path) { throw "Worktree path already exists: $path" }
  git worktree add $path -b $branch $BaseSha
  Write-Host "Created $path on $branch at $BaseSha"
  Write-Host "Assign unique PORT/DB/TMP/cache namespaces before running the agent."
} finally { Pop-Location }
