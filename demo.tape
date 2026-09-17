# Visual Settings
Output demo.gif
Set Shell "bash"
Set FontSize 15
Set FontFamily "JetBrains Mono"
Set Width 1050
Set Height 520
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 10
Set Padding 20
Set TypingSpeed 60ms

# Ensure PATH preserves system binaries (/usr/bin, /bin) alongside portopsy
Hide
Type "export PATH=\(HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:\)PATH"
Enter
Type "clear"
Enter
Show

# 1. Standard usage & Colored TUI output
Type "portopsy 3000"
Sleep 400ms
Enter
Sleep 3.5s

# 2. Flexible syntax (:3000) with --no-color flag
Type "clear"
Enter
Sleep 300ms
Type "portopsy --no-color :3000"
Sleep 400ms
Enter
Sleep 3s

# 3. AI Agent / Machine-readable JSON output
Type "clear"
Enter
Sleep 300ms
Type "portopsy 3000 --json"
Sleep 400ms
Enter
Sleep 3.5s

# 4. Interactive Resolver Menu (--resolve)
Type "clear"
Enter
Sleep 300ms
Type "portopsy 3000 --resolve"
Sleep 400ms
Enter
Sleep 2.5s

# Select option [4] Cancel to exit safely
Type "4"
Sleep 500ms
Enter
Sleep 2s
