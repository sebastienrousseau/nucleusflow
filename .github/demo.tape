# VHS tape recipe for recording NucleusFlow demo
# Output: .github/demo.gif
Output .github/demo.gif

Set Shell "bash"
Set FontSize 15
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Catppuccin Mocha"

Hide
Type "export PS1='$ ' && clear"
Enter
Show

Type "nucleusflow --help"
Enter
Sleep 3s

Type "cargo run --example basic"
Enter
Sleep 4s
