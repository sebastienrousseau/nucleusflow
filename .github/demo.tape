# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Rendered by `vhs` (https://github.com/charmbracelet/vhs) from the repository root.
# It demonstrates the NucleusFlow CLI lifecycle: scaffolding a project,
# inspecting structure, building static assets, and showing generated HTML.
Output .github/demo.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 640
Set Padding 20
Set Theme "Catppuccin Mocha"

Hide
Type `export PATH="$PWD/target/release:$PATH" && cd "$(mktemp -d /tmp/nucleusflow-demo.XXXX)" && export PS1='$ ' && clear`
Enter
Show

Type "nucleusflow --help"
Sleep 500ms
Enter
Sleep 3s

Type "nucleusflow new blog"
Sleep 500ms
Enter
Sleep 3s

Type "cd blog && ls -la content templates"
Sleep 500ms
Enter
Sleep 3s

Type "nucleusflow build --content content --templates templates --output public"
Sleep 500ms
Enter
Sleep 3s

Type "head -n 25 public/index.html"
Sleep 500ms
Enter
Sleep 4s
