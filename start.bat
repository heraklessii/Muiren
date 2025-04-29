@echo off
chcp 65001 >nul
title Muiren
echo Bot başlatılıyor...
:main
node .
echo ÇÖKME UYARISI... Bot çöktü.
goto main
