Use 
```
xargo rustc --release --target i686-pc-windows-msvc -- --emit=obj
xargo rustc --release --target x86_64-pc-windows-msvc -- --emit=obj
```
to build the code outputing the obj file that can be used as an inbut to a linker ( or crinkler )


then do 
```
..\..\..\..\..\tools\crinkler /OUT:test.exe /SUBSYSTEM:WINDOWS miniwin.o  /ENTRY:mainCRTStartup "/LIBPATH:C:\Program Files (x86)\Windows Kits\10\Lib\10.0.17763.0\um\x86" gdi32.lib user32.lib opengl32.lib kernel32.lib
```
to compress it with crinkler