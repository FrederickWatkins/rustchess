# unchess

A rusty chess library, CLI and hopefully eventually engine, TUI and maybe even GUI.

## Examples

### unchess-lib
```rust
use unchess_lib::{game::*, board::*, traits::*, types::*};

fn main() {
    let mut g = GameTree::<TransparentBoard>::starting_board();
    println!("{}", g.current_board());
    match g.disambiguate_move(AmbiguousMove::try_from("e4").unwrap()) {
        Ok(chess_move) => g.move_piece_checked(chess_move).unwrap(),
        Err(e) => println!("{e}"),
    }
    println!("{}", g.current_board());
}
```

### unchess-cli
```shell-session
foo@bar:~$ unchess-cli
8 r n b q k b n r
7 p p p p p p p p
6 ☐   ☐   ☐   ☐  
5   ☐   ☐   ☐   ☐
4 ☐   ☐   ☐   ☐  
3   ☐   ☐   ☐   ☐
2 P P P P P P P P
1 R N B Q K B N R
  a b c d e f g h 
e4
8 r n b q k b n r
7 p p p p p p p p
6 ☐   ☐   ☐   ☐  
5   ☐   ☐   ☐   ☐
4 ☐   ☐   P   ☐  
3   ☐   ☐   ☐   ☐
2 P P P P ☐ P P P
1 R N B Q K B N R
  a b c d e f g h 
d5
8 r n b q k b n r
7 p p p ☐ p p p p
6 ☐   ☐   ☐   ☐  
5   ☐   p   ☐   ☐
4 ☐   ☐   P   ☐  
3   ☐   ☐   ☐   ☐
2 P P P P ☐ P P P
1 R N B Q K B N R
  a b c d e f g h 
xd5
8 r n b q k b n r
7 p p p ☐ p p p p
6 ☐   ☐   ☐   ☐  
5   ☐   P   ☐   ☐
4 ☐   ☐   ☐   ☐  
3   ☐   ☐   ☐   ☐
2 P P P P ☐ P P P
1 R N B Q K B N R
  a b c d e f g h 
```