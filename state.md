Unimplemented.

- Evaluating for loops
- Defining variables after evaluating if statements causes a parser error. (BUG!!)
- The modulus operator also does not work.
- Decide in whether or not pointers will be exposed in the language.
- Get the main function to execute as a standalone bin file.
- Should we have a postfix syntax
  ```
    name++
  ```
- Perform multivariable declaration on a single line.
  ```go
    // The equivalent of var name1, name2 = ...
	```

- Perform mulitvariable reassignment on a single line.

# Syntax and Semantics.

## Struct Type

Defining abstract data types i think. This defines certain constructs like structs and enums.

```ksm-lang
struct Foo {
  var _name1 Type
  var _name2 Type

  fn baz() {} 
}
```

The definition of the syntax of our struct types brings into question 
how to expose defined methods to the broader compilation unit.
Which scopes will the user defined types be visible to and how will we denote that visibility.

- pub, private??

How much control can one have when specifying this?
Obviously not everything needs to live in the global scope. But also not everything specified within a
limited scope needs to be constrained to all modules.

Note: Struct types can be opaque types. This means they can be used to contain recursive types.

## Switch case statements

We must have nice things.
We'll go about designing our on semantics of switch case statements and leverage the LLVM IR's 
indirect_br instruction.
The behavior ought to match one's expectation if they were doing it from a different language.


## 22/07/2026

So currently i am looking at the opportunity that has fallen onto my lap.
We have llvm, mlir and circt.

This means i can do programming language design, look into how certain hardware targets can be 
compiled to and also look into circuit design.

--- These offer room for careful inspection of the underlying infrastructure ---

The first task though is rather simple.
- Build the language.
- Build the backend to spade emitting verilog.
- Swap the backend and use mlir.
In all the second ad third instances use the opportunity to use circt as a guide.
