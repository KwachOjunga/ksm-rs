# Struct Declaration Plan

The following design document strives to elaborate the manner 
in which structs are declared and implemented in kisumu lang.

Typical struct def without fields:
```ksm-lang
struct Struct_Ident {}
```

Struct def with fields of values and defined methods:

```ksm-lang
struct Struct_Ident {
    field_ident: Type_keyword,
    ...

    fn fn_ident() {
        return 
    }
    ....
}
```

Struct def with methods but no value fields though with defined methods.

```ksm-lang
struct Struct_Ident {
    fn fn_ident() {
        return 
    }
    ....
}
```


From the three snippets above, there are a few things one is inclined to observe.
First the definition of a struct type will be terminated by a '}' without being terminated by a ';'.
This means for one the parsing of the struct def will be primarily inclined towards determining 
the type implemented within the namespace of the struct.


By and large, the struct def in LLVM IR translates to something similar to the following:


```llvm
%Struct_Ident = type { i32, f64, %struct_ident2 }
%struct_ident2 = type { u32, f64 }
```
This is equivalent to the following kisumu-lang struct definition:

```ksm-lang
struct struct_ident2 {
    field1 u32,
    field2 f64,
};

struct Struct_Ident {
    field1 i32,
    field2 f64,
    field3 struct_ident2,
};
```

The nested struct fields should have an avenue to be defined without tied to an explicit external definition
as above.

```ksm-lang
struct struct_ident2 {
    field1 u32,
    field2 f64,
    field3 {field1 u32, field2 f64},
};
```

To support the struct declaration in the compiler implementation, the ast needs
extending to have struct type as a first class type in the ast.
