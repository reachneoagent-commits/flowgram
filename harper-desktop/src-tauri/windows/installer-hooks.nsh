; Gramflow keeps running in the tray and runs a second "highlighter" process from the same
; executable. Windows cannot delete a running program, so the uninstaller (and an upgrade)
; must stop both first, otherwise the files stay behind and the app looks un-uninstalled.

!macro NSIS_HOOK_PREINSTALL
  nsExec::Exec 'taskkill /F /T /IM "${MAINBINARYNAME}.exe"'
  Pop $0
  Sleep 800
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'taskkill /F /T /IM "${MAINBINARYNAME}.exe"'
  Pop $0
  Sleep 800
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove the "start at login" entry so a removed app does not try to launch itself.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${MAINBINARYNAME}"
!macroend
