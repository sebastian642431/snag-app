@echo off
title Snag console
cd /d "%~dp0"

set "YTDLP=yt-dlp"
where yt-dlp >nul 2>&1 || set "YTDLP=%LOCALAPPDATA%\Microsoft\WinGet\Packages\yt-dlp.yt-dlp_Microsoft.Winget.Source_8wekyb3d8bbwe\yt-dlp.exe"

set "OUT=%USERPROFILE%\Downloads\yt-dlp"

echo ============================================================
echo   Files go to: %OUT%
echo ============================================================

:loop
echo.
set "URL="
set /p "URL=Paste a link (empty Enter to quit): "
if not defined URL goto :done

set "MODE="
set /p "MODE=1 = MP3    2 = Video    [1]: "
if not defined MODE set "MODE=1"

echo.
if "%MODE%"=="2" (
  "%YTDLP%" --no-playlist -f "bv*+ba/b" --merge-output-format mp4 -o "%OUT%\%%(title)s.%%(ext)s" "%URL%"
) else (
  "%YTDLP%" --no-playlist -x --audio-format mp3 --audio-quality 0 -o "%OUT%\%%(title)s.%%(ext)s" "%URL%"
)

echo.
echo ------------------------------------------------------------
goto :loop

:done
echo.
echo Done. Files are in %OUT%
pause >nul
