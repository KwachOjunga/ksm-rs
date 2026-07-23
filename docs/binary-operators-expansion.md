# Binary Operators Expansion

The current implementation that supports the set of possible binary operators
is scant and fails to address most of the ways one might want to act on a pair of bits.

The proposal is to update the supported binary operators to include the following options:

-   BinaryOperator::Add 
-   BinaryOperator::Sub 
-   BinaryOperator::Mul 
-   BinaryOperator::Div 
-   BinaryOperator::Mod 
-   BinaryOperator::Eq ,
-   BinaryOperator::Neq ,
-   BinaryOperator::Lt 
-   BinaryOperator::Gt 
-   BinaryOperator::Le 
-   BinaryOperator::Ge 
-   BinaryOperator::LogicalAnd 
-   BinaryOperator::LogicalOr 
-   BinaryOperator::LogicalXor 
-   BinaryOperator::LeftShift 
-   BinaryOperator::RightShift 
-   BinaryOperator::ArithmeticRightShift 
-   BinaryOperator::BitwiseAnd 
-   BinaryOperator::BitwiseOr 
-   BinaryOperator::BitwiseXor 
-   BinaryOperator::WrappingAdd 
-   BinaryOperator::WrappingSub 
-   BinaryOperator::WrappingMul 
-   BinaryOperator::WrappingLeftShift 
-   BinaryOperator::WrappingRightShift 

Following the previous efforts of updating the ast accordingly,
the `BinOp` defined in the ast will be updated to reflect new binops
and the parsers also updated to act on resolving expressions with their 
respective symbols.
