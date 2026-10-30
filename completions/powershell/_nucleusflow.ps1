# PowerShell completion for nucleusflow
# SPDX-License-Identifier: Apache-2.0 OR MIT

Register-ArgumentCompleter -Native -CommandName nucleusflow -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        $commandElements |
        Select-Object -ExpandProperty Value |
        Where-Object { $_ -and -not ($_ -like "-*") }
    )

    if ($command.Count -le 1) {
        $subcommands = @("new", "build", "serve", "help")
        $subcommands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
        return
    }
}
