# fish completion for nucleusflow
# SPDX-License-Identifier: Apache-2.0 OR MIT

complete -c nucleusflow -s v -l verbose -d "Enable verbose logging"
complete -c nucleusflow -s h -l help -d "Print help"
complete -c nucleusflow -s V -l version -d "Print version"

# Subcommands
complete -c nucleusflow -n "__fish_use_subcommand" -a new -d "Create a new static site project"
complete -c nucleusflow -n "__fish_use_subcommand" -a build -d "Build the static site"
complete -c nucleusflow -n "__fish_use_subcommand" -a serve -d "Start the development server"

# Command: new
complete -c nucleusflow -n "__fish_seen_subcommand_from new" -s t -l template -x -a "blog docs portfolio" -d "Template to use"
complete -c nucleusflow -n "__fish_seen_subcommand_from new" -s h -l help -d "Print help"

# Command: build
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s c -l content-dir -r -d "Path to content directory"
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s o -l output-dir -r -d "Path to output directory"
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s t -l template-dir -r -d "Path to template directory"
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s m -l minify -d "Enable minification of output files"
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s f -l config -r -d "Build configuration file"
complete -c nucleusflow -n "__fish_seen_subcommand_from build" -s h -l help -d "Print help"

# Command: serve
complete -c nucleusflow -n "__fish_seen_subcommand_from serve" -s p -l port -x -d "Port to serve on"
complete -c nucleusflow -n "__fish_seen_subcommand_from serve" -s w -l watch -d "Enable file watching"
complete -c nucleusflow -n "__fish_seen_subcommand_from serve" -s d -l dir -r -d "Base directory to serve from"
complete -c nucleusflow -n "__fish_seen_subcommand_from serve" -s h -l help -d "Print help"
