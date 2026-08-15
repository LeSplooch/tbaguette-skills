: << 'CMDBLOCK'
@echo off
REM Cross-platform polyglot wrapper for TBaguette's hook scripts.
REM
REM On native Windows: cmd.exe runs this batch portion, which locates a
REM bash (Git for Windows in its standard install locations, or bash
REM already on PATH) and hands off to it.
REM On macOS/Linux/WSL: this whole file is a normal bash script. The batch
REM header above is inert there -- ":" is bash's no-op builtin, so this
REM block does nothing and execution falls through to the Unix section
REM below.
REM
REM Hook scripts use extensionless filenames (e.g. "session-start", not
REM "session-start.sh") on purpose -- Claude Code's Windows auto-detection
REM prepends "bash" to any command containing ".sh", which would collide
REM with this wrapper already doing that.
REM
REM Usage: run-hook.cmd <script-name> [args...]

if "%~1"=="" (
    echo run-hook.cmd: missing script name >&2
    exit /b 1
)

set "HOOK_DIR=%~dp0"

if exist "C:\Program Files\Git\bin\bash.exe" (
    "C:\Program Files\Git\bin\bash.exe" "%HOOK_DIR%%~1" %2 %3 %4 %5 %6 %7 %8 %9
    exit /b %ERRORLEVEL%
)
if exist "C:\Program Files (x86)\Git\bin\bash.exe" (
    "C:\Program Files (x86)\Git\bin\bash.exe" "%HOOK_DIR%%~1" %2 %3 %4 %5 %6 %7 %8 %9
    exit /b %ERRORLEVEL%
)

where bash >nul 2>nul
if %ERRORLEVEL% equ 0 (
    bash "%HOOK_DIR%%~1" %2 %3 %4 %5 %6 %7 %8 %9
    exit /b %ERRORLEVEL%
)

REM No bash found -- exit silently rather than error. The plugin still
REM works, just without SessionStart context injection on this machine.
exit /b 0
CMDBLOCK

# Unix: run the named hook script directly.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT_NAME="$1"
shift
exec bash "${SCRIPT_DIR}/${SCRIPT_NAME}" "$@"
