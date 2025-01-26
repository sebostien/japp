# JAPP

A semi-functional programming language inspired by Agda.

**Example:**

Print 5 factorial

```agda
infixl _-_  1 ;
infixl _*_  2 ;

fac : i32 -> i32 ;
fn fac 0 = 1 ;
fn fac n = n * fac (n - 1) ;

fn main = console.log(fac(5)) ;
```


## TODO:

- [x] Lexer
- [ ] Spanned errors in expr parser.
- [ ] Blocks in body '{ ...; ...; ... }'
- [ ] Error recovery (parse until next ';' and try again).
- [ ] Idents should specify number of args: `_+_` or `!_`
- [ ] Mixfix `if_then_else_`
  - [ ] Parser for mixfix
  - [ ] ExprParser handles mixfix
- [ ] Types
  - [ ] Trivial: bool, int, float
  - [ ] String
  - [ ] Reference/Pointer
  - [ ] Refined: Option<T>
  - [ ] Typechecking, De brujin or something
