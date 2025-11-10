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
    // 改行を空白に置換
    let cleaned = s.replace('\n', " ").replace('\r', "");

    // 特殊文字を含むかチェック（;, *, (), [], {}, など）
    let needs_quotes = cleaned.chars().any(|c| {
        matches!(c, ';' | '*' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':')
    });

    if needs_quotes {
        // 二重引用符で囲む（ラベル内の二重引用符は削除）
        format!("\"{}\"", cleaned.replace('"', ""))
    } else {
        cleaned
    }
}
