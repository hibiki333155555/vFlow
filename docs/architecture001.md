# vFlow Architecture

## プロジェクト概要
vFlowは、C言語のソースコードから制御フロー図を自動生成するツールです。

## システムアーキテクチャ

```mermaid
flowchart TB
    User[ユーザー] -->|コマンド実行| CLI[main.rs<br/>CLIエントリーポイント]
    CLI -->|引数解析| Input{入力判定}
    Input -->|ディレクトリ| ProcDir[process_directory<br/>ディレクトリ内の.cファイルを処理]
    Input -->|単一ファイル| ProcFile[process_single_file<br/>単一ファイルを処理]

    ProcDir -->|各.cファイル| ProcFile
    ProcFile -->|ファイル読込| Parser[parser.rs<br/>parse_c_code]

    Parser -->|tree-sitter解析| AST[抽象構文木<br/>AST生成]
    AST -->|関数抽出| Functions[Function構造体<br/>name + body]

    Functions -->|各関数| CFGBuilder[cfg.rs<br/>build_cfg]
    CFGBuilder -->|CFG構築| CFG[ControlFlowGraph<br/>nodes + edges]

    CFG -->|レンダリング| Renderer[renderer.rs<br/>render_mermaid]
    Renderer -->|Mermaid形式| Output[出力ファイル<br/>.md]
    Output -->|保存| Result[結果<br/>制御フロー図]
```

## メイン処理フロー

```mermaid
flowchart TD
    Start([開始]) --> ParseArgs[コマンドライン引数パース]
    ParseArgs --> ReadFile[Cファイル読込]
    ReadFile --> Parse[C言語パース<br/>tree-sitter]

    Parse --> ExtractFunc[関数定義抽出]
    ExtractFunc --> ExtractStmt[ステートメント抽出<br/>Simple/If]

    ExtractStmt --> LoopFunc{全関数処理?}
    LoopFunc -->|次の関数| BuildCFG[CFG構築]

    BuildCFG --> CreateEntry[Entry/Exitノード作成]
    CreateEntry --> ProcessStmt[ステートメント処理]

    ProcessStmt --> CheckType{ステートメント型}
    CheckType -->|Simple| AddSimple[Simpleノード追加]
    CheckType -->|If| AddCond[Conditionノード追加]

    AddSimple --> NextStmt{次のステートメント}
    AddCond --> AddBranch[Then/Else分岐追加]
    AddBranch --> NextStmt

    NextStmt -->|あり| ProcessStmt
    NextStmt -->|なし| Optimize[空ノード削除<br/>ID再マッピング]

    Optimize --> RenderMermaid[Mermaid形式レンダリング]
    RenderMermaid --> LoopFunc

    LoopFunc -->|完了| WriteFile[ファイル書込]
    WriteFile --> End([終了])
```

## モジュール構成

```mermaid
flowchart LR
    subgraph "cfg-generator"
        Main[main.rs<br/>エントリーポイント]
        Lib[lib.rs<br/>公開API]

        subgraph "コアモジュール"
            Parser[parser.rs<br/>C言語パーサー]
            CFG[cfg.rs<br/>CFG構築器]
            Renderer[renderer.rs<br/>Mermaidレンダラー]
        end
    end

    Main --> Lib
    Lib --> Parser
    Lib --> CFG
    Lib --> Renderer

    Parser -->|Function/Statement| CFG
    CFG -->|ControlFlowGraph| Renderer
```

## データ構造

```mermaid
classDiagram
    class Function {
        +String name
        +Vec~Statement~ body
    }

    class Statement {
        <<enumeration>>
        Simple
        If
    }

    class Simple {
        +String code
        +usize line
    }

    class If {
        +String condition
        +Vec~Statement~ then_branch
        +Option~Vec~Statement~~ else_branch
        +usize line
    }

    class ControlFlowGraph {
        +Vec~CFGNode~ nodes
        +Vec~CFGEdge~ edges
        +usize entry_id
        +usize exit_id
    }

    class CFGNode {
        +usize id
        +NodeType node_type
        +String label
    }

    class NodeType {
        <<enumeration>>
        Entry
        Exit
        Simple
        Condition
    }

    class CFGEdge {
        +usize from
        +usize to
        +Option~String~ label
    }

    Function *-- Statement
    Statement <|-- Simple
    Statement <|-- If
    ControlFlowGraph *-- CFGNode
    ControlFlowGraph *-- CFGEdge
    CFGNode *-- NodeType
```

## 処理の詳細

### 1. パース処理 (parser.rs)
- tree-sitterを使用してC言語の構文解析
- 関数定義を検出し、関数名と本体を抽出
- ステートメントをSimple（通常文）とIf（条件文）に分類

### 2. CFG構築 (cfg.rs)
- 関数ごとにEntry/Exitノードを作成
- ステートメントを順番に処理してノードとエッジを追加
- If文では条件ノードと分岐（true/false）を作成
- 空のノードを削除し、IDを最適化

### 3. レンダリング (renderer.rs)
- CFGをMermaid flowchart形式に変換
- ノード型に応じて形状を決定:
  - Entry/Exit: 丸角四角形 `([label])`
  - Condition: ひし形 `{label}`
  - Simple: 四角形 `[label]`
- エッジにラベル（true/false）を付加

## 入出力例

### 入力 (example.c)
```c
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

### 出力 (example.md)
- 関数maxの制御フローを表現するMermaidフローチャート
- Entry → Condition(a > b) → Then/Else分岐 → Exit

## 使用技術
- **言語**: Rust
- **パーサー**: tree-sitter (tree-sitter-c)
- **CLI**: clap
- **出力形式**: Mermaid flowchart
