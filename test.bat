@echo off
echo Script cuz i was too lazy to auto build n update shit myself cuz lazy
if %USERNAME%==siddh set mvn=C:\Users\siddh\apache-maven-3.8.6\bin\mvn.cmd
Rem If your maven install is broken like mine and u wanna have to specify the path, uncommend the line below and set ur mvn
Rem if %USERNAME%==uzair set mvn=C:\Users\uzair\mvn
cd dev.skidpacker.loader-jni/target/release
cargo build --release
cd ../../../dev.skidpacker.loader/src/main/resources/win32-x86-64
Copy ..\..\..\..\..\dev.skidpacker.loader-jni\target\release\loader_jni.dll
cd ..\..\..\..\..\
cmd.exe /c %mvn% -pl dev.skidpacker.loader,dev.skidpacker.testjar -am clean package
cd dev.skidpacker.encrypt-rust/target/release
Copy ..\..\..\dev.skidpacker.testjar\target\testjar-1.0-SNAPSHOT.jar
cmd.exe /c skidencrypt.exe -t -T 16 -i testjar-1.0-SNAPSHOT.jar -n THISISANONCE
cd ..\..\..\tests
Copy ..\dev.skidpacker.loader\target\dev.skidpacker.loader-1.0-SNAPSHOT.jar
Copy ..\dev.skidpacker.encrypt-rust\target\release\output.jar


set RUST_BACKTRACE=full
echo Running jar...
java -jar dev.skidpacker.loader-1.0-SNAPSHOT.jar
echo Run complete!