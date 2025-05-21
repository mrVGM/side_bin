@echo off

cmake -G "Ninja" -S . out || goto :error
cmake --build out || goto :error
copy out\compile_commands.json || goto :error

echo SUCCESS
exit(0)

echo FAIL
exit(1)

