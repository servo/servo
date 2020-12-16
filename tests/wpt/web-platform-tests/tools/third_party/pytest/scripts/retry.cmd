@echo off
rem Source: https://github.com/appveyor/ci/blob/master/scripts/appveyor-retry.cmd
rem initiate the retry number
set retryNumber=0
set maxRetries=3

:RUN
%*
set LastErrorLevel=%ERRORLEVEL%
IF %LastErrorLevel% == 0 GOTO :EOF
set /a retryNumber=%retryNumber%+1
IF %reTryNumber% == %maxRetries% (GOTO :FAILED)

:RETRY
set /a retryNumberDisp=%retryNumber%+1
@echo Command "%*" failed with exit code %LastErrorLevel%. Retrying %retryNumberDisp% of %maxRetries%
GOTO :RUN

: FAILED
@echo Sorry, we tried running command for %maxRetries% times and all attempts were unsuccessful!
EXIT /B %LastErrorLevel%
