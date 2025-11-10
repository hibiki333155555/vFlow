# 制御フロー図生成ツール MVP設計書

## 概要
C言語のソースコードから制御フロー図（CFG）を生成し、Mermaid形式で出力する最小限の実装。
初期バージョンでは順次実行とif文のみをサポート。

## スコープ（MVP - 1-2日で実装）

### サポートする構文
- ✅ 順次実行（単純文）
- ✅ if文（else含む）
- ✅ 関数の開始・終了
- ❌ ループ（for/while）- Phase 2
- ❌ switch文 - Phase 2
- ❌ break/continue/goto - Phase 2

## プロジェクト構造

```
cfg-generator/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLIエントリポイント
│   ├── lib.rs           # ライブラリルート
│   ├── parser.rs        # パーサーモジュール（tree-sitter使用）
│   ├── cfg.rs           # CFG構造とビルダー
│   └── renderer.rs      # Mermaid出力
└── tests/
    └── integration_test.rs
```

## 依存関係（Cargo.toml）

```toml
[package]
name = "cfg-generator"
version = "0.1.0"
edition = "2021"

[dependencies]
tree-sitter = "0.20"
tree-sitter-c = "0.20"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"

[dev-dependencies]
pretty_assertions = "1.3"
```

## 実装詳細

### 1. メイン関数（main.rs）

```rust
use anyhow::Result;
use clap::Parser;
use cfg_generator::{parse_c_code, build_cfg, render_mermaid};

#[derive(Parser)]
#[command(name = "cfg-gen")]
#[command(about = "C言語の制御フロー図生成ツール")]
struct Cli {
    /// 入力Cファイル
    input: String,
    
    /// 出力ファイル（未指定時は標準出力）
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // ファイル読み込み
    let source = std::fs::read_to_string(&cli.input)?;
    
    // パース
    let functions = parse_c_code(&source)?;
    
    // 各関数のCFG生成
    let mut output = String::new();
    for func in functions {
        let cfg = build_cfg(func)?;
        output.push_str(&render_mermaid(&cfg));
        output.push_str("\n");
    }
    
    // 出力
    if let Some(out_path) = cli.output {
        std::fs::write(out_path, output)?;
    } else {
        print!("{}", output);
    }
    
    Ok(())
}
```

### 2. パーサー（parser.rs）

```rust
use anyhow::{Result, Context};
use tree_sitter::{Parser, Node};

pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Simple {
        code: String,
        line: usize,
    },
    If {
        condition: String,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        line: usize,
    },
}

pub fn parse_c_code(source: &str) -> Result<Vec<Function>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_c::language())
        .context("Failed to load C grammar")?;
    
    let tree = parser.parse(source, None)
        .context("Failed to parse source code")?;
    
    let root = tree.root_node();
    let mut functions = Vec::new();
    
    // 関数定義を探す
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(func) = parse_function(child, source) {
                functions.push(func);
            }
        }
    }
    
    Ok(functions)
}

fn parse_function(node: Node, source: &str) -> Option<Function> {
    // 関数名を取得
    let name = get_function_name(node, source)?;
    
    // 関数本体を取得
    let body_node = node.child_by_field_name("body")?;
    let body = parse_compound_statement(body_node, source);
    
    Some(Function { name, body })
}

fn get_function_name(node: Node, source: &str) -> Option<String> {
    let declarator = node.child_by_field_name("declarator")?;
    let identifier = find_identifier(declarator)?;
    Some(source[identifier.byte_range()].to_string())
}

fn find_identifier(node: Node) -> Option<Node> {
    if node.kind() == "identifier" {
        return Some(node);
    }
    
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(id) = find_identifier(child) {
            return Some(id);
        }
    }
    None
}

fn parse_compound_statement(node: Node, source: &str) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        match child.kind() {
            "if_statement" => {
                if let Some(stmt) = parse_if_statement(child, source) {
                    statements.push(stmt);
                }
            }
            "expression_statement" | 
            "declaration" | 
            "return_statement" => {
                let code = source[child.byte_range()].trim().to_string();
                let line = child.start_position().row + 1;
                statements.push(Statement::Simple { code, line });
            }
            "compound_statement" => {
                // ネストしたブロック
                let nested = parse_compound_statement(child, source);
                statements.extend(nested);
            }
            _ => {}
        }
    }
    
    statements
}

fn parse_if_statement(node: Node, source: &str) -> Option<Statement> {
    let condition_node = node.child_by_field_name("condition")?;
    let condition = source[condition_node.byte_range()].to_string();
    
    let consequence = node.child_by_field_name("consequence")?;
    let then_branch = if consequence.kind() == "compound_statement" {
        parse_compound_statement(consequence, source)
    } else {
        vec![Statement::Simple {
            code: source[consequence.byte_range()].to_string(),
            line: consequence.start_position().row + 1,
        }]
    };
    
    let else_branch = node.child_by_field_name("alternative").map(|alt| {
        if alt.kind() == "compound_statement" {
            parse_compound_statement(alt, source)
        } else if alt.kind() == "if_statement" {
            // else if の場合
            vec![parse_if_statement(alt, source).unwrap()]
        } else {
            vec![Statement::Simple {
                code: source[alt.byte_range()].to_string(),
                line: alt.start_position().row + 1,
            }]
        }
    });
    
    Some(Statement::If {
        condition,
        then_branch,
        else_branch,
        line: node.start_position().row + 1,
    })
}
```

### 3. CFG構造とビルダー（cfg.rs）

```rust
use crate::parser::{Function, Statement};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CFGNode {
    pub id: usize,
    pub node_type: NodeType,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Entry,
    Exit,
    Simple,
    Condition,
}

#[derive(Debug, Clone)]
pub struct CFGEdge {
    pub from: usize,
    pub to: usize,
    pub label: Option<String>, // "true", "false", or None
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub nodes: Vec<CFGNode>,
    pub edges: Vec<CFGEdge>,
    pub entry_id: usize,
    pub exit_id: usize,
}

pub struct CFGBuilder {
    next_id: usize,
    nodes: Vec<CFGNode>,
    edges: Vec<CFGEdge>,
}

impl CFGBuilder {
    fn new() -> Self {
        Self {
            next_id: 0,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    fn add_node(&mut self, node_type: NodeType, label: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(CFGNode { id, node_type, label });
        id
    }
    
    fn add_edge(&mut self, from: usize, to: usize, label: Option<String>) {
        self.edges.push(CFGEdge { from, to, label });
    }
    
    fn build_statements(&mut self, statements: &[Statement], entry: usize, exit: usize) {
        if statements.is_empty() {
            self.add_edge(entry, exit, None);
            return;
        }
        
        let mut current = entry;
        
        for (i, stmt) in statements.iter().enumerate() {
            match stmt {
                Statement::Simple { code, .. } => {
                    let node_id = self.add_node(NodeType::Simple, code.clone());
                    self.add_edge(current, node_id, None);
                    current = node_id;
                    
                    // 最後の文の場合はexitに接続
                    if i == statements.len() - 1 {
                        self.add_edge(current, exit, None);
                    }
                }
                Statement::If { condition, then_branch, else_branch, .. } => {
                    // 条件ノードを作成
                    let cond_id = self.add_node(NodeType::Condition, condition.clone());
                    self.add_edge(current, cond_id, None);
                    
                    // 合流点を作成
                    let merge_id = if i == statements.len() - 1 {
                        exit
                    } else {
                        self.add_node(NodeType::Simple, "".to_string())
                    };
                    
                    // then分岐
                    if then_branch.is_empty() {
                        self.add_edge(cond_id, merge_id, Some("true".to_string()));
                    } else {
                        let then_entry = self.add_node(NodeType::Simple, "".to_string());
                        self.add_edge(cond_id, then_entry, Some("true".to_string()));
                        self.build_statements(then_branch, then_entry, merge_id);
                    }
                    
                    // else分岐
                    if let Some(else_stmts) = else_branch {
                        if else_stmts.is_empty() {
                            self.add_edge(cond_id, merge_id, Some("false".to_string()));
                        } else {
                            let else_entry = self.add_node(NodeType::Simple, "".to_string());
                            self.add_edge(cond_id, else_entry, Some("false".to_string()));
                            self.build_statements(else_stmts, else_entry, merge_id);
                        }
                    } else {
                        self.add_edge(cond_id, merge_id, Some("false".to_string()));
                    }
                    
                    current = merge_id;
                }
            }
        }
    }
    
    fn build(self, entry_id: usize, exit_id: usize) -> ControlFlowGraph {
        // 空のノードを削除（ラベルが空の単純ノード）
        let nodes: Vec<_> = self.nodes.into_iter()
            .filter(|n| !n.label.is_empty() || n.node_type != NodeType::Simple)
            .collect();
            
        ControlFlowGraph {
            nodes,
            edges: self.edges,
            entry_id,
            exit_id,
        }
    }
}

pub fn build_cfg(function: Function) -> Result<ControlFlowGraph> {
    let mut builder = CFGBuilder::new();
    
    // Entry/Exitノードを作成
    let entry = builder.add_node(NodeType::Entry, format!("START: {}", function.name));
    let exit = builder.add_node(NodeType::Exit, format!("END: {}", function.name));
    
    // 関数本体のCFGを構築
    builder.build_statements(&function.body, entry, exit);
    
    Ok(builder.build(entry, exit))
}
```

### 4. Mermaidレンダラー（renderer.rs）

```rust
use crate::cfg::{ControlFlowGraph, NodeType};

pub fn render_mermaid(cfg: &ControlFlowGraph) -> String {
    let mut output = String::new();
    output.push_str("```mermaid\nflowchart TD\n");
    
    // ノードの描画
    for node in &cfg.nodes {
        let shape = match node.node_type {
            NodeType::Entry | NodeType::Exit => {
                format!("    {}([{}])\n", node.id, escape_mermaid(&node.label))
            }
            NodeType::Condition => {
                format!("    {}{{{}}}\n", node.id, escape_mermaid(&node.label))
            }
            NodeType::Simple => {
                if !node.label.is_empty() {
                    format!("    {}[{}]\n", node.id, escape_mermaid(&node.label))
                } else {
                    String::new() // 空のノードは描画しない
                }
            }
        };
        output.push_str(&shape);
    }
    
    // エッジの描画
    for edge in &cfg.edges {
        let arrow = if let Some(label) = &edge.label {
            format!("    {} -->|{}| {}\n", edge.from, label, edge.to)
        } else {
            format!("    {} --> {}\n", edge.from, edge.to)
        };
        output.push_str(&arrow);
    }
    
    output.push_str("```\n");
    output
}

fn escape_mermaid(s: &str) -> String {
    s.replace('"', "&quot;")
     .replace('\'', "&apos;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('\n', " ")
     .replace('\r', "")
}
```

### 5. ライブラリエントリポイント（lib.rs）

```rust
mod parser;
mod cfg;
mod renderer;

pub use parser::{parse_c_code, Function, Statement};
pub use cfg::{build_cfg, ControlFlowGraph};
pub use renderer::render_mermaid;
```

## テストコード例

### tests/integration_test.rs

```rust
use cfg_generator::{parse_c_code, build_cfg, render_mermaid};

#[test]
fn test_simple_if() {
    let source = r#"
        int max(int a, int b) {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
    "#;
    
    let functions = parse_c_code(source).unwrap();
    assert_eq!(functions.len(), 1);
    
    let cfg = build_cfg(functions.into_iter().next().unwrap()).unwrap();
    let output = render_mermaid(&cfg);
    
    assert!(output.contains("START: max"));
    assert!(output.contains("END: max"));
    assert!(output.contains("a > b"));
    assert!(output.contains("true"));
    assert!(output.contains("false"));
}

#[test]
fn test_sequential_statements() {
    let source = r#"
        void process() {
            int x = 10;
            x = x + 1;
            printf("%d\n", x);
        }
    "#;
    
    let functions = parse_c_code(source).unwrap();
    let cfg = build_cfg(functions.into_iter().next().unwrap()).unwrap();
    let output = render_mermaid(&cfg);
    
    println!("{}", output); // デバッグ用出力
    
    assert!(output.contains("int x = 10"));
    assert!(output.contains("x = x + 1"));
    assert!(output.contains("printf"));
}
```

## 使用例

### 入力ファイル（example.c）

```c
int factorial(int n) {
    if (n <= 1) {
        return 1;
    } else {
        int result = n * factorial(n - 1);
        return result;
    }
}
```

### コマンド実行

```bash
# ビルド
cargo build --release

# 実行（標準出力）
./target/release/cfg-gen example.c

# ファイルに出力
./target/release/cfg-gen example.c -o output.md
```

### 期待される出力

```mermaid
flowchart TD
    0([START: factorial])
    1{n <= 1}
    2[return 1;]
    3[int result = n * factorial(n - 1);]
    4[return result;]
    5([END: factorial])
    
    0 --> 1
    1 -->|true| 2
    1 -->|false| 3
    2 --> 5
    3 --> 4
    4 --> 5
```

## 実装のポイント

1. **エラー処理を簡潔に**
   - MVPではpanicを避け、`Result`を使用
   - ただし、詳細なエラーメッセージは後回し

2. **最小限の機能に集中**
   - 複雑な最適化は行わない
   - 空のノードの削除程度に留める

3. **テスト駆動開発**
   - 簡単なテストケースから始める
   - 出力を目視確認できるようにする

4. **段階的な拡張を意識**
   - 後からループやswitch文を追加しやすい構造
   - ビルダーパターンで拡張性を確保

## 次のステップ（Phase 2以降）

- [ ] while/forループのサポート
- [ ] break/continueの処理
- [ ] switch-case文
- [ ] 関数呼び出しの可視化
- [ ] Graphviz形式の出力
- [ ] 基本ブロックの最適化
- [ ] 循環的複雑度の計算

## デバッグのコツ

1. **tree-sitterのAST確認**
```rust
// デバッグ用: ASTを出力
fn print_ast(node: Node, source: &str, depth: usize) {
    println!("{}{}: {}", "  ".repeat(depth), node.kind(), 
             &source[node.byte_range()].chars().take(20).collect::<String>());
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_ast(child, source, depth + 1);
    }
}
```

2. **中間結果の確認**
   - CFG構築前後でノード数を確認
   - エッジの接続を図示して確認

このMVP設計により、1-2日で基本的な制御フロー図生成ツールが実装可能です。