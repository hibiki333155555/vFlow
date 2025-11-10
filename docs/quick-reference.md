# クイックリファレンス

vFlowの開発・学習時によく使うコマンドやコードパターンをまとめたリファレンスです。

## 📋 目次
1. [コマンドライン](#コマンドライン)
2. [Rustの基本パターン](#rustの基本パターン)
3. [tree-sitterの操作](#tree-sitterの操作)
4. [デバッグテクニック](#デバッグテクニック)
5. [よくあるエラーと対処法](#よくあるエラーと対処法)

---

## コマンドライン

### ビルド

```bash
# デバッグビルド（開発中）
cargo build

# リリースビルド（最適化あり）
cargo build --release

# ビルドとテストを同時に
cargo build --release && cargo test
```

### 実行

```bash
# デフォルト実行（example_code/ディレクトリを処理）
cargo run --release

# または
./target/release/cfg-generator

# 単一ファイルを指定
cargo run --release example_code/example.c -o output/test.md

# または
./target/release/cfg-generator example_code/example.c -o output/test.md

# ヘルプを表示
cargo run --release -- --help
./target/release/cfg-generator --help
```

### テスト

```bash
# 全テストを実行
cargo test

# 詳細な出力で実行
cargo test -- --nocapture

# 特定のテストのみ実行
cargo test test_simple_function

# テストと出力を同時に確認
cargo test -- --nocapture --test-threads=1
```

### ドキュメント生成

```bash
# プロジェクトのドキュメントを生成して開く
cargo doc --open

# 依存クレートのドキュメントも含める
cargo doc --open --document-private-items
```

### コードフォーマットとリント

```bash
# コードを自動フォーマット
cargo fmt

# フォーマットチェックのみ
cargo fmt -- --check

# リントチェック
cargo clippy

# より厳密なリント
cargo clippy -- -W clippy::pedantic
```

---

## Rustの基本パターン

### エラーハンドリング

#### Result型の使用

```rust
// ?演算子でエラーを伝搬
fn my_function() -> Result<String> {
    let content = fs::read_to_string("file.txt")?;
    Ok(content)
}

// contextでエラーメッセージを追加
use anyhow::Context;

let tree = parser.parse(source, None)
    .context("Failed to parse source code")?;
```

#### Option型の使用

```rust
// ?演算子でNoneを即座にreturn
fn get_name(node: Node, source: &str) -> Option<String> {
    let identifier = node.child_by_field_name("declarator")?;
    Some(source[identifier.byte_range()].to_string())
}

// unwrap_or_defaultでデフォルト値を使用
let name = maybe_name.unwrap_or_default();

// if letで値がある場合のみ処理
if let Some(name) = maybe_name {
    println!("Name: {}", name);
}
```

### イテレータ

```rust
// for-inループ
for child in node.children(&mut cursor) {
    println!("{}", child.kind());
}

// filter + map + collect
let simple_stmts: Vec<_> = statements.iter()
    .filter(|s| matches!(s, Statement::Simple { .. }))
    .map(|s| s.code())
    .collect();

// enumerate（インデックス付き）
for (i, stmt) in statements.iter().enumerate() {
    println!("Statement {}: {:?}", i, stmt);
}
```

### パターンマッチング

```rust
// 列挙型のマッチ
match stmt {
    Statement::Simple { code, line } => {
        println!("Simple at line {}: {}", line, code);
    }
    Statement::If { condition, .. } => {
        println!("If condition: {}", condition);
    }
}

// if letで特定のパターンのみ処理
if let Statement::If { condition, then_branch, .. } = stmt {
    // If文の場合のみ処理
}

// matchesマクロ
if matches!(stmt, Statement::Simple { .. }) {
    // Simpleの場合のみtrue
}
```

### コレクション操作

```rust
// Vecの作成
let mut vec = Vec::new();
vec.push(item);

// 要素を追加
vec.extend(other_vec);

// 長さチェック
if vec.is_empty() {
    // 空の場合
}

// HashMapの使用
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert(key, value);

if let Some(value) = map.get(&key) {
    // 値が存在する場合
}
```

---

## tree-sitterの操作

### パーサーのセットアップ

```rust
use tree_sitter::{Parser, Node};

let mut parser = Parser::new();
parser.set_language(tree_sitter_c::language())
    .expect("Error loading C grammar");

let tree = parser.parse(source_code, None)
    .expect("Error parsing");

let root = tree.root_node();
```

### ノードの走査

```rust
// 全ての子ノードを走査
let mut cursor = node.walk();
for child in node.children(&mut cursor) {
    println!("Child kind: {}", child.kind());
}

// 特定のフィールドを取得
if let Some(body) = node.child_by_field_name("body") {
    // bodyノードが存在する場合
}

// 再帰的な探索
fn find_all_identifiers(node: Node) -> Vec<Node> {
    let mut result = Vec::new();

    if node.kind() == "identifier" {
        result.push(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        result.extend(find_all_identifiers(child));
    }

    result
}
```

### ノードの情報取得

```rust
// ノードの種類
let kind = node.kind();

// ソースコードの取得
let text = &source[node.byte_range()];

// 位置情報
let start = node.start_position();
let line = start.row + 1;  // 行番号（1始まり）
let column = start.column; // 列番号（0始まり）

// 子ノードの数
let child_count = node.child_count();

// 親ノード（Optionで返される）
if let Some(parent) = node.parent() {
    // 親が存在する場合
}
```

### よく使うノード種別

```rust
match node.kind() {
    "function_definition" => { /* 関数定義 */ }
    "if_statement" => { /* if文 */ }
    "compound_statement" => { /* ブロック */ }
    "expression_statement" => { /* 式文 */ }
    "declaration" => { /* 変数宣言 */ }
    "return_statement" => { /* return文 */ }
    "identifier" => { /* 識別子 */ }
    "parenthesized_expression" => { /* 括弧付き式 */ }
    _ => { /* その他 */ }
}
```

---

## デバッグテクニック

### 1. println!デバッグ

```rust
// 関数の入口
println!("[ENTER] parse_function: {}", &source[node.byte_range()]);

// 変数の値を確認
println!("[DEBUG] name = {}, body.len() = {}", name, body.len());

// ループの進行状況
for (i, stmt) in statements.iter().enumerate() {
    println!("[LOOP] Processing statement {}/{}", i + 1, statements.len());
}

// 関数の出口
println!("[EXIT] parse_function -> {:?}", result);
```

### 2. dbg!マクロ

```rust
// 値を表示しつつ、その値を返す
let name = dbg!(get_function_name(node, source)?);

// 複数の値を確認
dbg!(&name, &body.len(), &node.kind());

// 式の途中で使用
let result = dbg!(a + b) * 2;
```

### 3. ASTの可視化

```rust
// ノードの構造を再帰的に表示
fn print_ast(node: Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = &source[node.byte_range()];
    let preview = if text.len() > 40 {
        format!("{}...", &text[..40])
    } else {
        text.to_string()
    };

    println!("{}{}: {}", indent, node.kind(), preview.replace('\n', "\\n"));

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_ast(child, source, depth + 1);
    }
}

// 使用例
print_ast(root, source, 0);
```

### 4. CFGの可視化

```rust
// CFGの構造を表示
impl ControlFlowGraph {
    pub fn print_debug(&self) {
        println!("=== CFG Debug ===");
        println!("Entry: {}, Exit: {}", self.entry_id, self.exit_id);

        println!("\nNodes:");
        for node in &self.nodes {
            println!("  [{}] {:?}: {}", node.id, node.node_type, node.label);
        }

        println!("\nEdges:");
        for edge in &self.edges {
            let label = edge.label.as_deref().unwrap_or("");
            println!("  {} --{}-> {}", edge.from, label, edge.to);
        }
    }
}
```

### 5. テストの追加

```rust
#[test]
fn test_my_feature() {
    let source = r#"
        int test(int x) {
            if (x > 0) {
                return 1;
            }
            return 0;
        }
    "#;

    let functions = parse_c_code(source).unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "test");

    let cfg = build_cfg(functions[0].clone()).unwrap();
    println!("{:#?}", cfg);  // 詳細表示
}
```

---

## よくあるエラーと対処法

### コンパイルエラー

#### 借用チェッカーエラー

```rust
// エラー: valueのムーブ後に使用
let s = String::from("hello");
let t = s;  // sからtへムーブ
println!("{}", s);  // エラー！

// 解決策1: clone()でコピー
let s = String::from("hello");
let t = s.clone();
println!("{}", s);  // OK

// 解決策2: 参照を使う
let s = String::from("hello");
let t = &s;
println!("{}", s);  // OK
```

#### 可変借用と不変借用の競合

```rust
// エラー: 不変借用中に可変借用
let mut vec = vec![1, 2, 3];
let first = &vec[0];
vec.push(4);  // エラー！
println!("{}", first);

// 解決策: 借用のスコープを分ける
let mut vec = vec![1, 2, 3];
{
    let first = &vec[0];
    println!("{}", first);
}  // firstのスコープ終了
vec.push(4);  // OK
```

### 実行時エラー

#### パースエラー

```
Error: Failed to parse source code
```

**原因**: tree-sitterが文法エラーを検出
**対処**: 入力のC言語コードを確認

#### ノード取得の失敗

```rust
// child_by_field_nameがNoneを返す
let body = node.child_by_field_name("body")
    .expect("body not found");  // panic!
```

**対処**: Option型を適切に処理

```rust
if let Some(body) = node.child_by_field_name("body") {
    // bodyが存在する場合のみ処理
} else {
    eprintln!("Warning: body not found for {}", node.kind());
}
```

### 論理エラー

#### 空のCFGが生成される

**原因**: ステートメントの抽出に失敗
**デバッグ**: `parse_compound_statement`にprintln!を追加

```rust
fn parse_compound_statement(node: Node, source: &str) -> Vec<Statement> {
    println!("[DEBUG] Parsing compound_statement with {} children", node.child_count());
    // ...
}
```

#### エッジが正しく接続されない

**原因**: CFG構築時のロジックミス
**デバッグ**: CFGの中間状態を表示

```rust
println!("[DEBUG] Adding edge: {} -> {}, label: {:?}",
         from, to, label);
```

---

## ファイル構成チートシート

```
vFlow/
├── cfg-generator/
│   ├── src/
│   │   ├── main.rs         # エントリーポイント、CLI処理
│   │   ├── lib.rs          # 公開API
│   │   ├── parser.rs       # C言語パーサー（tree-sitter）
│   │   ├── cfg.rs          # CFG構築ロジック
│   │   └── renderer.rs     # Mermaid形式レンダリング
│   ├── tests/
│   │   └── integration_test.rs  # 統合テスト
│   └── Cargo.toml          # 依存関係
├── example_code/           # サンプルCファイル
├── output/                 # 生成されたMermaid図
├── docs/                   # ドキュメント
│   ├── learning-guide.md
│   ├── fundamental-concepts.md
│   └── quick-reference.md
├── architecture.md         # アーキテクチャ図
└── README.md               # プロジェクト概要
```

---

## 便利なVSCode拡張機能

開発を快適にする拡張機能:

1. **rust-analyzer**: Rustの補完、型チェック
2. **Error Lens**: エラーをインラインで表示
3. **CodeLLDB**: デバッガ
4. **Markdown All in One**: Mermaid図のプレビュー
5. **Mermaid Markdown Syntax Highlighting**: Mermaid構文のハイライト

---

## 次のステップ

このリファレンスで基本操作に慣れたら、以下に挑戦:

1. **新しいテストケースを追加**: `tests/integration_test.rs`
2. **エラーメッセージを改善**: より分かりやすいメッセージに
3. **新機能の実装**: while/forループのサポートなど

困ったときはこのリファレンスに戻ってきてください！
