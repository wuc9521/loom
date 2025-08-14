# LS-Parser API Framework

这个API框架允许用户定义自定义的数据类型（sum types），并自动派生 `Syntax` trait 以与LS-Parser系统集成。

## 核心概念

### Syntax Trait

`Syntax` trait 是所有用户定义的数据类型必须实现的核心接口。它提供了：

- **SMT-LIB格式转换**: `to_smtlib()` 和 `from_smtlib()`
- **类型检查**: `validate()`
- **元数据访问**: `name()`, `sort()`, `constructor_count()`, `constructor_name()`

### 编译时检查

框架使用编译时检查确保用户实现了所有必要的函数。如果缺少实现，编译器会显示清晰的错误消息。

## 使用方法

### 1. 基本用法

```rust
use ls_parser::api::Syntax;

#[derive(Debug, Clone)]
enum MyList {
    Nil,
    Cons(i32, Box<MyList>),
}

// 使用宏实现基本的Syntax功能
ls_parser::impl_syntax_basic!(MyList);

// 实现必要的函数
impl MyList {
    fn to_smtlib(&self) -> String {
        match self {
            MyList::Nil => "nil".to_string(),
            MyList::Cons(head, tail) => format!("(cons {} {})", head, tail.to_smtlib()),
        }
    }
    
    fn from_smtlib(s: &str) -> Result<Self, ls_parser::api::SyntaxError> {
        // 实现解析逻辑
        todo!()
    }
    
    fn validate(&self) -> Result<(), ls_parser::api::SyntaxError> {
        Ok(())
    }
    
    fn constructor_count(&self) -> usize {
        2 // Nil 和 Cons
    }
    
    fn constructor_name(&self, index: usize) -> Option<&str> {
        match index {
            0 => Some("Nil"),
            1 => Some("Cons"),
            _ => None,
        }
    }
}
```

### 2. 编译时检查

```rust
use ls_parser::api::Syntax;

#[derive(Debug, Clone)]
enum BadList {
    Nil,
    Cons(i32, Box<BadList>),
}

// 这会生成编译时检查
ls_parser::syntax_check!(BadList);

// 如果用户没有实现必要的函数，编译器会报错：
// error[E0599]: no function or associated item named `to_smtlib` found for struct `BadList`
// error[E0599]: no function or associated item named `from_smtlib` found for struct `BadList`
// error[E0599]: no function or associated item named `validate` found for struct `BadList`
```

### 3. 便利宏

```rust
// 组合基本实现和编译时检查
ls_parser::derive_syntax!(MyType);
```

## 必需函数

用户必须实现以下函数：

1. **`to_smtlib(&self) -> String`**: 将值转换为SMT-LIB格式
2. **`from_smtlib(s: &str) -> Result<Self, SyntaxError>`**: 从SMT-LIB格式解析值
3. **`validate(&self) -> Result<(), SyntaxError>`**: 验证值的正确性
4. **`constructor_count(&self) -> usize`**: 返回构造函数的数量
5. **`constructor_name(&self, index: usize) -> Option<&str>`**: 返回指定索引的构造函数名称

## 错误处理

框架使用 `SyntaxError` 枚举来处理错误：

```rust
pub enum SyntaxError {
    ParseError(String),
    ValidationError(String),
    TypeError(String),
    Unsupported(String),
}
```

## 示例

### 正则表达式数据类型

```rust
#[derive(Debug, Clone)]
enum RE {
    Empty,
    Eps,
    Sym(char),
    Union(Box<RE>, Box<RE>),
    Concat(Box<RE>, Box<RE>),
    Star(Box<RE>),
}

ls_parser::impl_syntax_basic!(RE);

impl RE {
    fn to_smtlib(&self) -> String {
        match self {
            RE::Empty => "empty".to_string(),
            RE::Eps => "eps".to_string(),
            RE::Sym(c) => format!("(sym {})", c),
            RE::Union(l, r) => format!("(union {} {})", l.to_smtlib(), r.to_smtlib()),
            RE::Concat(l, r) => format!("(concat {} {})", l.to_smtlib(), r.to_smtlib()),
            RE::Star(b) => format!("(star {})", b.to_smtlib()),
        }
    }
    
    // ... 其他实现
}
```

### 字符枚举

```rust
#[derive(Debug, Clone)]
enum Char {
    A,
    B,
}

ls_parser::impl_syntax_basic!(Char);

impl Char {
    fn to_smtlib(&self) -> String {
        match self {
            Char::A => "a".to_string(),
            Char::B => "b".to_string(),
        }
    }
    
    // ... 其他实现
}
```

## 与LS-Parser集成

实现 `Syntax` trait 的数据类型可以：

1. **自动序列化**: 转换为SMT-LIB格式用于求解器
2. **类型检查**: 在编译时验证类型正确性
3. **元数据访问**: 获取类型信息和构造函数
4. **错误处理**: 统一的错误处理机制

## 最佳实践

1. **始终实现所有必需函数**: 使用编译时检查确保完整性
2. **提供有意义的错误消息**: 在 `from_smtlib` 和 `validate` 中返回有用的错误
3. **保持一致性**: 确保 `to_smtlib` 和 `from_smtlib` 是互逆的
4. **验证输入**: 在 `validate` 中检查数据结构的完整性
5. **文档化**: 为复杂的数据类型提供清晰的文档

## 测试

运行测试来验证框架功能：

```bash
# 运行基本功能测试
cargo run --bin macro_test

# 运行编译错误演示
cargo run --bin compile_error_test

# 运行API示例
cargo run --bin api_test
```
