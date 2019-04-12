# RustBCL
BCL in Rust

Global Pointer:
Apr. 12:
  Done:
    split Config and GlobalPointer;
    change all memory related value to usize, void\* pointer to \*u8;

Apr. 11:
  Done:
    Config: a struct for "global" variable;
    GlobalPointer: new, rget, rput;
  TODO:
    malloc part