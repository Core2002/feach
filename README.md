
<h1 align="center">📁 FEach</h1>
<h5 align="center">This tool can foreach dirs and run command.</h5>

------

Build:  
`cargo build --release`  

Usage:  
```shell
feach> .\target\release\feach.exe -h
Usage: feach.exe [OPTIONS] --cmd <CMD>

Options:
  -d, --dir <DIR>        [default: ./]
  -c, --cmd <CMD>
  -l, --layers <LAYERS>  [default: 1]
  -h, --help             Print help
  -V, --version          Print version
```

E.g:  
`feach -c "git pull"`  
`feach -c "cmd /c tree"`
