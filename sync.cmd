@echo off
rem Pull, commit everything, push - in one go. Usage:  sync "what changed"
setlocal
set "MSG=%~1"
if "%MSG%"=="" set "MSG=Update"
git pull --no-rebase --no-edit
if errorlevel 1 (
  echo.
  echo Pull stopped on a conflict. Nothing was pushed - tell Claude.
  exit /b 1
)
git add -A
git diff --cached --quiet
if errorlevel 1 git commit -m "%MSG%"
git push
