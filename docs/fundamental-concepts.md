# 基礎知識詳細解説

このドキュメントでは、vFlowを理解するために必要な基礎知識を詳しく解説します。

## 目次
1. [抽象構文木（AST）](#抽象構文木ast)
2. [制御フローグラフ（CFG）](#制御フローグラフcfg)
3. [tree-sitterの基礎](#tree-sitterの基礎)
4. [Rustの重要な概念](#rustの重要な概念)
5. [グラフ理論の基礎](#グラフ理論の基礎)

---

## 抽象構文木（AST）

### ASTとは

**抽象構文木（Abstract Syntax Tree）** は、プログラムの構文構造をツリー形式で表現したデータ構造です。

### なぜ「抽象」なのか

元のソースコードから以下のような「具体的な」情報を省略しているため：
- 括弧の位置
- セミコロンの位置
- インデント
- コメント

### 具体例

**C言語のコード:**
```c
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

**対応するAST（簡略版）:**
```
function_definition
├── type: int
├── declarator
│   ├── identifier: max
│   └── parameters
│       ├── parameter: int a
│       └── parameter: int b
└── body (compound_statement)
    └── if_statement
        ├── condition: a > b
        ├── consequence
        │   └── return_statement: a
        └── alternative
            └── return_statement: b
```

### ASTの構成要素

1. **ルートノード**: プログラム全体またはファイル全体を表す
2. **内部ノード**: 構文要素（if文、関数定義など）
3. **リーフノード**: 識別子、リテラル（数値、文字列など）

### ASTの利点

- **構文チェック**: 文法的に正しいかを判定
- **意味解析**: コードの意味を理解
- **コード変換**: リファクタリング、最適化
- **コード生成**: 別の言語やバイトコードへの変換

---

## 制御フローグラフ（CFG）

### CFGとは

**制御フローグラフ（Control Flow Graph）** は、プログラムの実行順序を表現する有向グラフです。

### CFGの構成要素

#### ノード（Node）
プログラムの基本ブロックを表す。基本ブロックとは、分岐や合流がない一連の命令群。

**ノードの種類:**
- **Entry**: プログラムや関数の開始点
- **Exit**: プログラムや関数の終了点
- **Simple**: 通常の処理ブロック
- **Condition**: 分岐条件の評価

#### エッジ（Edge）
ノード間の制御の流れを表す有向辺。

**エッジの種類:**
- **無条件エッジ**: 必ず次のノードへ進む
- **条件付きエッジ**: 条件によって分岐する（true/false）

### 具体例

**シンプルなコード:**
```c
int f(int x) {
    int y = x + 1;
    return y;
}
```

**CFG:**
```
[Entry: START f]
      ↓
[Simple: int y = x + 1;]
      ↓
[Simple: return y;]
      ↓
[Exit: END f]
```

**分岐を含むコード:**
```c
int sign(int x) {
    if (x > 0) {
        return 1;
    } else {
        return -1;
    }
}
```

**CFG:**
```
[Entry: START sign]
      ↓
{Condition: x > 0}
   /           \
true         false
 /               \
[return 1]    [return -1]
 \               /
  \             /
   [Exit: END sign]
```

### CFGの用途

1. **コード最適化**: 到達不可能なコード（デッドコード）の検出
2. **静的解析**: バグやセキュリティ脆弱性の検出
3. **テストカバレッジ**: どのパスがテストされたか追跡
4. **コード理解**: プログラムの振る舞いの可視化

### CFGの特性

- **有向グラフ**: エッジに方向がある
- **連結グラフ**: 全てのノードがEntryから到達可能
- **単一Entry、単一Exit**: 通常、1つの入口と1つの出口

---

## tree-sitterの基礎

### tree-sitterとは

**tree-sitter** は、高速でインクリメンタルな構文解析器生成ツールです。

### 特徴

1. **高速**: 大規模なファイルも素早く解析
2. **インクリメンタル**: ファイルの一部が変更されても全体を再解析不要
3. **エラー耐性**: 構文エラーがあっても部分的に解析可能
4. **多言語対応**: C、Rust、Python、JavaScriptなど多数

### 基本的な使い方

```rust
use tree_sitter::{Parser, Node};

// 1. パーサーを作成
let mut parser = Parser::new();

// 2. 言語文法をセット
parser.set_language(tree_sitter_c::language())
    .expect("Error loading C grammar");

// 3. ソースコードをパース
let source_code = "int main() { return 0; }";
let tree = parser.parse(source_code, None)
    .expect("Error parsing");

// 4. ASTのルートノードを取得
let root_node = tree.root_node();
```

### ノードの操作

#### ノードの種類を取得
```rust
let kind = node.kind();  // "function_definition", "if_statement" など
```

#### 子ノードの取得
```rust
// 特定のフィールド名で取得
let body = node.child_by_field_name("body");

// 全ての子ノードを走査
let mut cursor = node.walk();
for child in node.children(&mut cursor) {
    // childに対する処理
}
```

#### ノードの位置情報
```rust
let start = node.start_position();  // 開始位置（行、列）
let end = node.end_position();      // 終了位置
let byte_range = node.byte_range(); // バイト範囲
```

#### ソースコードの取得
```rust
let text = &source_code[node.byte_range()];
```

### vFlowでの使用例

`parser.rs`の`parse_c_code`関数:

```rust
pub fn parse_c_code(source: &str) -> Result<Vec<Function>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_c::language())?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    let mut functions = Vec::new();
    let mut cursor = root.walk();

    // 関数定義を探す
    for child in root.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(func) = parse_function(child, source) {
                functions.push(func);
            }
        }
    }

    Ok(functions)
}
```

### C言語の主なノード種別

| ノード種別 | 説明 | 例 |
|-----------|------|-----|
| `function_definition` | 関数定義 | `int max(int a, int b) { ... }` |
| `if_statement` | if文 | `if (x > 0) { ... }` |
| `compound_statement` | 複合文（ブロック） | `{ ... }` |
| `expression_statement` | 式文 | `x = 5;` |
| `declaration` | 変数宣言 | `int x;` |
| `return_statement` | return文 | `return 0;` |
| `identifier` | 識別子 | `max`, `x`, `a` |

---

## Rustの重要な概念

vFlowのコードを理解するために必要なRustの概念を解説します。

### 1. 所有権と借用

#### 所有権（Ownership）
- 各値には唯一の所有者がある
- 所有者がスコープを抜けると値は破棄される

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1からs2に所有権が移動（ムーブ）
// println!("{}", s1);  // エラー！s1はもう使えない
println!("{}", s2);  // OK
```

#### 借用（Borrowing）
- 所有権を移さずに参照を渡す
- `&`で不変参照、`&mut`で可変参照

```rust
fn print_length(s: &String) {  // 借用
    println!("{}", s.len());
}

let s = String::from("hello");
print_length(&s);  // sの参照を渡す
println!("{}", s);  // まだsは使える
```

**vFlowでの例:**
```rust
fn parse_compound_statement(node: Node, source: &str) -> Vec<Statement>
//                                        ^^^^^ sourceを借用
```

### 2. Result型とエラーハンドリング

#### Result型
```rust
enum Result<T, E> {
    Ok(T),    // 成功時の値
    Err(E),   // エラー時の値
}
```

#### ?演算子
エラーを自動的に伝搬させる。

```rust
fn parse_c_code(source: &str) -> Result<Vec<Function>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_c::language())?;  // エラー時は即座にreturn
    //                                              ↑ ?演算子

    // 成功した場合のみここに到達
    let tree = parser.parse(source, None)?;
    // ...
}
```

**?演算子なしの場合:**
```rust
let tree = match parser.parse(source, None) {
    Ok(t) => t,
    Err(e) => return Err(e),
};
```

### 3. Option型

値があるかないかを表現する型。

```rust
enum Option<T> {
    Some(T),  // 値がある
    None,     // 値がない
}
```

**vFlowでの例:**
```rust
fn parse_function(node: Node, source: &str) -> Option<Function> {
    let name = get_function_name(node, source)?;  // Noneなら即座にreturn None
    let body_node = node.child_by_field_name("body")?;

    Some(Function { name, body })
}
```

### 4. パターンマッチ

`match`式で列挙型の値によって処理を分岐。

```rust
match child.kind() {
    "if_statement" => {
        // if文の処理
    }
    "expression_statement" => {
        // 式文の処理
    }
    _ => {
        // その他
    }
}
```

### 5. クロージャとイテレータ

**イテレータの例:**
```rust
for child in node.children(&mut cursor) {
    // childに対する処理
}
```

**collect()でコレクションを構築:**
```rust
let children: Vec<_> = condition_node.children(&mut cursor).collect();
```

### 6. 構造体とメソッド

```rust
pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
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
        // self を可変借用
    }
}
```

### 7. 列挙型（Enum）

```rust
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
```

**使用例:**
```rust
match stmt {
    Statement::Simple { code, line } => {
        // Simpleの場合の処理
    }
    Statement::If { condition, then_branch, else_branch, line } => {
        // Ifの場合の処理
    }
}
```

---

## グラフ理論の基礎

### グラフとは

**グラフ（Graph）** は、ノード（頂点）とエッジ（辺）の集合です。

```
記法: G = (V, E)
V: ノードの集合
E: エッジの集合
```

### グラフの種類

#### 有向グラフ（Directed Graph）
エッジに方向がある。

```
A → B → C
    ↓
    D
```

**CFGは有向グラフです。**

#### 無向グラフ（Undirected Graph）
エッジに方向がない。

```
A - B - C
    |
    D
```

### グラフの表現方法

#### 隣接リスト（vFlowの方式）
各ノードから出るエッジをリストで保持。

```rust
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,  // (from, to, label)
}
```

**利点:**
- メモリ効率が良い
- エッジの走査が速い

### グラフの操作

#### ノードの追加
```rust
fn add_node(&mut self, node_type: NodeType, label: String) -> usize {
    let id = self.next_id;
    self.next_id += 1;
    self.nodes.push(CFGNode { id, node_type, label });
    id
}
```

#### エッジの追加
```rust
fn add_edge(&mut self, from: usize, to: usize, label: Option<String>) {
    self.edges.push(CFGEdge { from, to, label });
}
```

### グラフのトラバース（走査）

#### 深さ優先探索（DFS: Depth-First Search）
一つのパスを最後まで辿ってから戻る。

```
     A
   /   \
  B     C
 / \     \
D   E     F

訪問順: A → B → D → E → C → F
```

#### 幅優先探索（BFS: Breadth-First Search）
同じ深さのノードを先に訪問。

```
訪問順: A → B → C → D → E → F
```

**vFlowでは明示的な走査は行いませんが、CFG構築時に暗黙的にDFS的な処理を行っています。**

### グラフの性質

#### 連結性
全てのノードが互いに到達可能かどうか。

CFGは通常連結グラフで、全てのノードがEntryから到達可能です。

#### サイクル（閉路）
始点に戻ってくるパス。

```
A → B → C → A
```

**vFlowの現在の実装では、ループ文を扱わないためサイクルはありません。**

---

## まとめ

### vFlowで使われている概念の対応表

| 概念 | vFlowでの使用箇所 | 重要度 |
|------|------------------|--------|
| AST | `parser.rs`全体 | ⭐⭐⭐ |
| CFG | `cfg.rs`全体 | ⭐⭐⭐ |
| tree-sitter | `parser.rs`の`parse_c_code` | ⭐⭐⭐ |
| 所有権と借用 | 全体 | ⭐⭐⭐ |
| Result型 | エラーハンドリング全般 | ⭐⭐⭐ |
| Option型 | `parser.rs`の返り値 | ⭐⭐⭐ |
| パターンマッチ | ステートメントの処理 | ⭐⭐⭐ |
| 列挙型 | `Statement`, `NodeType` | ⭐⭐⭐ |
| グラフ理論 | CFGの構造理解 | ⭐⭐ |
| イテレータ | ノードの走査 | ⭐⭐ |

### 学習の進め方

1. **まずは概要を理解**: 各概念の大まかな意味を把握
2. **実際のコードで確認**: vFlowのコードで使われている箇所を見る
3. **手を動かす**: デバッグ出力を追加したり、小さな変更を試す
4. **図を描く**: ASTやCFGを紙に描いて視覚化
5. **繰り返し**: 分からなくなったらこのドキュメントに戻る

焦らず、自分のペースで学習を進めてください！
