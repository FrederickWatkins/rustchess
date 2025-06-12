```ebnf
<pgn> ::= (<tag> <whitespace>)* (<move_number> <whitespace>* <move> <whitespace>+ <move> <whitespace>+)+ <result>
<move> ::= <piece>? <src_dest> <promotion>? <action>? | "O-O" | "O-O-O"
<src_dest> ::= <file>? <rank>? <dest> | <dest>
<dest> ::= <takes>? <square>
<square> ::= <file> <rank>
<file> ::= [a-h]
<rank> ::= [1-8]
<takes> ::= "x"
<promotion> ::= "=" <piece>
<piece> ::= "Q" | "K" | "N" | "R" | "B"
<action> ::= "+" | "#"
<result> ::= "1-0" | "0-1" | "1/2-1/2"
<tag> ::= "[" ([a-z] | [A-Z])+ " " "\"" ([a-z] | [A-Z] | "/" | "." | " " | "," | [0-9] | "-")+ "\"" "]"
<whitespace> ::= " " | "\n"
<move_number> ::= [0-9]+ "."
```