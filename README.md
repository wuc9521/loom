# Logic Learning Language Parser

Frontend of Logic Learning Problems

To run the project.

Install dependencies.

```bash
cargo install
```

Some examples

```bash
# First Order Logic
cargo run -- --input ./exp/fol/1.sl

# Propositional Modal Logic
cargo run -- --input ./exp/pml/1.sl

# Regular Expression
cargo run -- --input ./exp/re/1.sl 
```

## Theory 扩展机制（SMT-LIB 兼容）

本项目支持通过加载 SMT-LIB 官方格式的 theory 文件（S-Expression），自动扩展 parser 对 theory-specific 指令和符号的支持。

- 只需将类似如下内容保存为 .sexp 或 .smt2 文件：

```
(theory Ints
 :sorts ((Int 0))
 :funs ((NUMERAL Int)
        (- Int Int)
        (- Int Int Int :left-assoc)
        (+ Int Int Int :left-assoc)
        (* Int Int Int :left-assoc)
        (div Int Int Int :left-assoc)
        (mod Int Int Int)
        (abs Int Int)
        (<= Int Int Bool :chainable)
        (<  Int Int Bool :chainable)
        (>= Int Int Bool :chainable)
        (>  Int Int Bool :chainable)))
```

- parser 会自动解析这些文件，注册所有 theory-specific 的函数、类型、属性。
- 新增/修改 theory 只需编辑 theory 文件，无需更改 parser 代码。

详见 `src/lang/theory.rs`。
