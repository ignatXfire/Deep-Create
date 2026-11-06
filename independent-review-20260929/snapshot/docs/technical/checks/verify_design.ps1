[CmdletBinding()]
param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [switch]$SelfTest
)

# Read-only documentation checks. No application, provider or secret access.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$utf8 = [Text.UTF8Encoding]::new($false, $true)

function Get-MarkdownShape([string]$Text) {
    $visible = [Collections.Generic.List[string]]::new()
    $fenceCharacter = ''
    $fenceLength = 0
    foreach ($line in ($Text -split '\r?\n')) {
        $match = [regex]::Match($line, '^ {0,3}(`{3,}|~{3,})(.*)$')
        if ($fenceLength -eq 0) {
            if ($match.Success) {
                $fenceCharacter = $match.Groups[1].Value.Substring(0, 1)
                $fenceLength = $match.Groups[1].Value.Length
                $visible.Add('')
            } else { $visible.Add($line) }
        } else {
            if ($match.Success -and
                $match.Groups[1].Value.StartsWith($fenceCharacter) -and
                $match.Groups[1].Value.Length -ge $fenceLength -and
                [string]::IsNullOrWhiteSpace($match.Groups[2].Value)) {
                $fenceLength = 0
            }
            $visible.Add('')
        }
    }
    [pscustomobject]@{ Visible = ($visible -join "`n"); Balanced = ($fenceLength -eq 0) }
}

function Get-RowCells([string]$Line) {
    $body = $Line.Trim().TrimStart('|').TrimEnd('|')
    @($body -split '(?<!\\)\|' | ForEach-Object { $_.Trim() })
}

function Get-SetDelta([string[]]$Expected, [string[]]$Actual) {
    [pscustomobject]@{
        Missing = @($Expected | Where-Object { $_ -cnotin $Actual } | Sort-Object -Unique)
        Extra = @($Actual | Where-Object { $_ -cnotin $Expected } | Sort-Object -Unique)
        Duplicates = @($Actual | Group-Object -CaseSensitive | Where-Object Count -gt 1 | ForEach-Object Name)
    }
}

function Get-LocalTargets([string]$Text) {
    foreach ($m in [regex]::Matches($Text, '\]\((?<target><[^>]+>|[^)\r\n]+)\)')) {
        $target = $m.Groups['target'].Value.Trim().Trim('<', '>')
        if ($target -match '^[a-zA-Z][a-zA-Z0-9+.-]*:' -or $target.StartsWith('#')) { continue }
        $path = ($target -split '#', 2)[0]
        if ($path.Length -gt 0) { [Uri]::UnescapeDataString($path) }
    }
}

function Get-ResultShape([string[]]$Cells, [int]$ExpectedCount) {
    return ($Cells.Count -eq $ExpectedCount -and @($Cells | Where-Object { [string]::IsNullOrWhiteSpace($_) }).Count -eq 0)
}

function Get-ScenarioIds([string]$Text) {
    @([regex]::Matches($Text, '\bAC-[A-Z]+-\d{2}\b') | ForEach-Object { $_.Value } | Sort-Object -Unique)
}

if ($SelfTest) {
    $tests = [Collections.Generic.List[object]]::new()
    $tests.Add(@('balanced_fence', (Get-MarkdownShape "~~~text`nbody`n~~~").Balanced))
    $tests.Add(@('unclosed_fence', (-not (Get-MarkdownShape '~~~text').Balanced)))
    $tests.Add(@('fence_content_hidden', ((Get-MarkdownShape "~~~`nC-99`n~~~").Visible -notmatch 'C-99')))
    $tests.Add(@('short_closer_rejected', (-not (Get-MarkdownShape "~~~~`nx`n~~~").Balanced)))
    $tests.Add(@('wrong_closer_rejected', (-not (Get-MarkdownShape "~~~`nx`n``````").Balanced)))
    $tests.Add(@('table_cells', (Get-ResultShape @(Get-RowCells '| one | two | three |') 3)))
    $tests.Add(@('empty_cell_rejected', (-not (Get-ResultShape @(Get-RowCells '| one | | three |') 3))))
    $tests.Add(@('escaped_pipe', (@(Get-RowCells '| a\|b | c |').Count -eq 2)))
    $delta = Get-SetDelta @('C-01', 'C-02') @('C-01', 'C-01', 'C-03')
    $tests.Add(@('missing_id', ($delta.Missing -ccontains 'C-02')))
    $tests.Add(@('unknown_id', ($delta.Extra -ccontains 'C-03')))
    $tests.Add(@('duplicate_id', ($delta.Duplicates -ccontains 'C-01')))
    $targets = @(Get-LocalTargets '[a](./a.md) [b](https://example.org/) [c](#part) [d](./b%20c.md#part)')
    $tests.Add(@('local_links_only', ($targets.Count -eq 2 -and $targets[1] -ceq './b c.md')))
    $badUtf8Rejected = $false
    try { $null = $utf8.GetString([byte[]]@(0xc3, 0x28)) } catch { $badUtf8Rejected = $true }
    $tests.Add(@('invalid_utf8_rejected', $badUtf8Rejected))
    $scenarioIds = @(Get-ScenarioIds 'AC-XW-01, AC-XW-01; AC-STATE-09. AC-XW-001')
    $tests.Add(@('scenario_ids_unique', ($scenarioIds.Count -eq 2)))
    $tests.Add(@('scenario_ids_exact_width', ($scenarioIds -cnotcontains 'AC-XW-00')))
    $scenarioDelta = Get-SetDelta @('AC-XW-01', 'AC-CAL-01') $scenarioIds
    $tests.Add(@('missing_scenario_detected', ($scenarioDelta.Missing -ccontains 'AC-CAL-01')))
    $passed = @($tests | Where-Object { $_[1] }).Count
    [ordered]@{ mode = 'self_test'; total = $tests.Count; passed = $passed; failed = @($tests | Where-Object { -not $_[1] } | ForEach-Object { $_[0] }) } | ConvertTo-Json -Depth 4
    if ($passed -eq $tests.Count) { exit 0 } else { exit 1 }
}

$rootPath = (Resolve-Path -LiteralPath $Root).Path
$checks = [Collections.Generic.List[object]]::new()
function Add-Check([string]$Name, [bool]$Passed, $Detail) {
    $checks.Add([pscustomobject]@{ name = $Name; passed = $Passed; detail = $Detail })
}
$documents = @{}
$fileRecords = [Collections.Generic.List[object]]::new()
foreach ($number in @('00') + @(7..26 | ForEach-Object { '{0:D2}' -f $_ })) {
    $files = @(Get-ChildItem -LiteralPath $rootPath -File -Filter ($number + '_*.md'))
    Add-Check ('file_' + $number) ($files.Count -eq 1) @{ found = $files.Count }
    if ($files.Count -ne 1) { continue }
    $file = $files[0]
    try {
        $text = $utf8.GetString([IO.File]::ReadAllBytes($file.FullName)).TrimStart([char]0xfeff)
        $shape = Get-MarkdownShape $text
        $documents[$number] = [pscustomobject]@{ File = $file; Text = $text; Visible = $shape.Visible }
        Add-Check ('text_' + $number) ($shape.Balanced -and $text -notmatch '\x00|\uFFFD') @{ utf8 = $true; fences_balanced = $shape.Balanced }
        $fileRecords.Add([pscustomobject]@{ name = $file.Name; sha256 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() })
    } catch {
        Add-Check ('text_' + $number) $false @{ error = 'Cannot decode or inspect document as strict UTF-8.' }
    }
}

$sourceIds = @()
$matrixIds = @()
$supplementIds = @()
$contractIds = @()
$reviewIds = @()
$missingEvents = @()
$localLinkCount = 0
$invalidLinks = [Collections.Generic.List[string]]::new()
foreach ($doc in $documents.Values) {
    foreach ($target in @(Get-LocalTargets $doc.Visible)) {
        $localLinkCount++
        try {
            $resolved = [IO.Path]::GetFullPath((Join-Path $doc.File.DirectoryName $target))
            if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) { $invalidLinks.Add($doc.File.Name + ': ' + $target) }
        } catch { $invalidLinks.Add($doc.File.Name + ': invalid local path') }
    }
}
Add-Check 'local_file_links' ($invalidLinks.Count -eq 0) @{ examined = $localLinkCount; broken = @($invalidLinks) }

if ($documents.ContainsKey('07') -and $documents.ContainsKey('21') -and $documents.ContainsKey('19') -and $documents.ContainsKey('18')) {
    $idPattern = '[\u041B\u0418]-\d{2}[\u0430-\u044f]?'
    $sourceIds = @([regex]::Matches($documents['07'].Visible, ('(?m)^### (' + $idPattern + ')\s')) | ForEach-Object { $_.Groups[1].Value })
    $rows = @($documents['21'].Visible -split '\r?\n' | Where-Object { $_ -match ('^\| (' + $idPattern + ') \|') })
    $matrixIds = @($rows | ForEach-Object { @(Get-RowCells $_)[0] })
    $delta = Get-SetDelta $sourceIds $matrixIds
    Add-Check 'source_decisions_51' ($sourceIds.Count -eq 51 -and @($sourceIds | Sort-Object -Unique).Count -eq 51) @{ count = $sourceIds.Count }
    Add-Check 'decision_matrix_51_unique_exact' ($matrixIds.Count -eq 51 -and $delta.Missing.Count -eq 0 -and $delta.Extra.Count -eq 0 -and $delta.Duplicates.Count -eq 0) $delta
    $badRows = @($rows | Where-Object { -not (Get-ResultShape @(Get-RowCells $_) 5) })
    Add-Check 'decision_rows_nonempty_5_cells' ($badRows.Count -eq 0) @{ invalid_count = $badRows.Count }

    $supplementRows = @($documents['21'].Visible -split '\r?\n' | Where-Object { $_ -match '^\| \u0414-\d{2} \|' })
    $supplementIds = @($supplementRows | ForEach-Object { @(Get-RowCells $_)[0] })
    $expectedSupplements = @(1..10 | ForEach-Object { ([char]0x0414).ToString() + ('-{0:D2}' -f $_) })
    $supplementDelta = Get-SetDelta $expectedSupplements $supplementIds
    Add-Check 'supplements_10_unique_exact' ($supplementDelta.Missing.Count -eq 0 -and $supplementDelta.Extra.Count -eq 0 -and $supplementDelta.Duplicates.Count -eq 0) $supplementDelta
    Add-Check 'supplement_rows_nonempty_4_cells' (@($supplementRows | Where-Object { -not (Get-ResultShape @(Get-RowCells $_) 4) }).Count -eq 0) @{ count = $supplementRows.Count }

    $contractIds = @([regex]::Matches($documents['19'].Visible, '(?m)^### (C-\d{2})\s') | ForEach-Object { $_.Groups[1].Value })
    $expectedContracts = @(1..16 | ForEach-Object { 'C-{0:D2}' -f $_ })
    $contractDelta = Get-SetDelta $expectedContracts $contractIds
    Add-Check 'contracts_16_unique_exact' ($contractDelta.Missing.Count -eq 0 -and $contractDelta.Extra.Count -eq 0 -and $contractDelta.Duplicates.Count -eq 0) $contractDelta
    $references = @([regex]::Matches(($rows + $supplementRows -join "`n"), '\bC-\d{2}\b') | ForEach-Object { $_.Value } | Sort-Object -Unique)
    $referenceDelta = Get-SetDelta $expectedContracts $references
    Add-Check 'contract_references_cover_exact_set' ($referenceDelta.Missing.Count -eq 0 -and $referenceDelta.Extra.Count -eq 0) $referenceDelta

    $reviewRows = @($documents['21'].Visible -split '\r?\n' | Where-Object { $_ -match '^\| \u0420-\d{2} \|' })
    $reviewIds = @($reviewRows | ForEach-Object { @(Get-RowCells $_)[0] })
    if ($documents.ContainsKey('23') -and $documents.ContainsKey('24') -and $documents.ContainsKey('25') -and $documents.ContainsKey('26')) {
        $reviewSourceRows = @($documents['23'].Visible -split '\r?\n' | Where-Object { $_ -match '^\| \u0420-\d{2} \|' })
        $reviewSourceIds = @($reviewSourceRows | ForEach-Object { @(Get-RowCells $_)[0] })
        $expectedReviews = @(1..12 | ForEach-Object { ([char]0x0420).ToString() + ('-{0:D2}' -f $_) })
        $sourceReviewDelta = Get-SetDelta $expectedReviews $reviewSourceIds
        $reviewDelta = Get-SetDelta $reviewSourceIds $reviewIds
        Add-Check 'review_dispositions_12_unique_exact' ($sourceReviewDelta.Missing.Count -eq 0 -and $sourceReviewDelta.Extra.Count -eq 0 -and $sourceReviewDelta.Duplicates.Count -eq 0) $sourceReviewDelta
        Add-Check 'review_matrix_matches_dispositions' ($reviewDelta.Missing.Count -eq 0 -and $reviewDelta.Extra.Count -eq 0 -and $reviewDelta.Duplicates.Count -eq 0) $reviewDelta
        Add-Check 'review_rows_nonempty_5_cells' (@($reviewRows | Where-Object { -not (Get-ResultShape @(Get-RowCells $_) 5) }).Count -eq 0) @{ count = $reviewRows.Count }
        $referencedScenarios = @(Get-ScenarioIds ($reviewRows -join "`n"))
        $scenarioText = @('23', '24', '25', '26' | ForEach-Object { $documents[$_].Visible }) -join "`n"
        $documentedScenarios = @(Get-ScenarioIds $scenarioText)
        $missingScenarios = @($referencedScenarios | Where-Object { $_ -cnotin $documentedScenarios })
        Add-Check 'review_scenarios_present_in_contracts' ($referencedScenarios.Count -gt 0 -and $missingScenarios.Count -eq 0) @{ referenced = $referencedScenarios.Count; missing = $missingScenarios }
    } else {
        Add-Check 'review_contract_inputs_present' $false @{ required = @('23', '24', '25', '26') }
    }

    $eventsInMatrix = @($rows + $reviewRows | ForEach-Object {
        $cells = @(Get-RowCells $_)
        if ($cells.Count -eq 5) {
            [regex]::Matches($cells[3], '`([a-z][a-z0-9_]*_[a-z0-9_]+)`') | ForEach-Object { $_.Groups[1].Value }
        }
    } | Sort-Object -Unique)
    $catalogEvents = @([regex]::Matches($documents['18'].Visible, '(?m)^\| `([a-z][a-z0-9_]+)` \|') | ForEach-Object { $_.Groups[1].Value })
    $missingEvents = @($eventsInMatrix | Where-Object { $_ -cnotin $catalogEvents })
    Add-Check 'matrix_event_names_in_catalog' ($missingEvents.Count -eq 0) @{ checked = $eventsInMatrix.Count; missing = $missingEvents }
    Add-Check 'catalog_events_unique' (@($catalogEvents | Group-Object -CaseSensitive | Where-Object Count -gt 1).Count -eq 0) @{ count = $catalogEvents.Count }

    $knownObsolete = @(
        @{ key = '15'; pattern = 'handle_command\(CommandEnvelope\)' },
        @{ key = '16'; pattern = 'commit_transition\(ExpectedRows, TransitionPlan\)' },
        @{ key = '19'; pattern = '\| `parse_result_packet`[^\r\n]*(DuplicateField|Oversize)' }
    )
    $obsoleteCount = 0
    foreach ($rule in $knownObsolete) {
        if ($documents.ContainsKey($rule.key) -and $documents[$rule.key].Text -match $rule.pattern) { $obsoleteCount++ }
    }
    Add-Check 'known_corrected_contract_spellings' ($obsoleteCount -eq 0) @{ regressions = $obsoleteCount }
} else {
    Add-Check 'cross_reference_inputs_present' $false @{ required = @('07', '18', '19', '21') }
}

$failed = @($checks | Where-Object { -not $_.passed })
[ordered]@{
    mode = 'documentation_structure_only'
    checked_at_utc = [DateTime]::UtcNow.ToString('o')
    root = $rootPath
    verdict = $(if ($failed.Count -eq 0) { 'PASS' } else { 'FAIL' })
    checks_total = $checks.Count
    checks_passed = $checks.Count - $failed.Count
    summary = @{ documents = $documents.Count; source_decisions = $sourceIds.Count; matrix_decisions = $matrixIds.Count; supplement_groups = $supplementIds.Count; code_contracts = $contractIds.Count; review_dispositions = $reviewIds.Count; local_links = $localLinkCount }
    failed_checks = $failed
    input_files = @($fileRecords | Sort-Object name)
    checker_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    limitations = @('No product implementation tested.', 'No proof of semantic completeness or absence of vulnerabilities.', 'Link targets checked as files; URL contents and Markdown anchor destinations not checked.')
} | ConvertTo-Json -Depth 8
if ($failed.Count -eq 0) { exit 0 } else { exit 1 }
