@echo off
echo Script cuz i was too lazy to auto build n update shit myself cuz lazy
if %USERNAME%==siddh set mvn=C:\Users\siddh\apache-maven-3.8.6\bin\mvn.cmd
Rem If your maven install is broken like mine and u wanna have to specify the path, uncommend the line below and set ur mvn
Rem if %USERNAME%==docto set mvn=C:\Users\docto\mvn
cd dev.skidpacker.loader-jni/target/release
cargo build --release
cd ../../../dev.skidpacker.loader/src/main/resources/win32-x86-64
Copy ..\..\..\..\..\dev.skidpacker.loader-jni\target\release\loader_jni.dll
cd ../../../../../
cmd.exe /c %mvn% -pl dev.skidpacker.loader -am clean package
cd dev.skidpacker.loader/target/
java -jar dev.skidpacker.loader-1.0-SNAPSHOT.jar
PAUSE