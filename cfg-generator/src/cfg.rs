use crate::parser::{Function, Statement};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CFGNode {
    pub id: usize,
    pub node_type: NodeType,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
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

    fn build_statements_with_edge(&mut self, statements: &[Statement], exit: usize, from: usize, edge_label: Option<String>) {
        if statements.is_empty() {
            self.add_edge(from, exit, edge_label);
            return;
        }

        // 最初のノードを作成
        match &statements[0] {
            Statement::Simple { code, .. } => {
                let first_id = self.add_node(NodeType::Simple, code.clone());
                self.add_edge(from, first_id, edge_label);

                if statements.len() == 1 {
                    self.add_edge(first_id, exit, None);
                } else {
                    self.build_statements(&statements[1..], first_id, exit);
                }
            }
            Statement::If { condition, then_branch, else_branch, .. } => {
                let cond_id = self.add_node(NodeType::Condition, condition.clone());
                self.add_edge(from, cond_id, edge_label);

                let merge_id = if statements.len() == 1 {
                    exit
                } else {
                    self.add_node(NodeType::Simple, "".to_string())
                };

                // then分岐
                if then_branch.is_empty() {
                    self.add_edge(cond_id, merge_id, Some("true".to_string()));
                } else {
                    self.build_statements_with_edge(then_branch, merge_id, cond_id, Some("true".to_string()));
                }

                // else分岐
                if let Some(else_stmts) = else_branch {
                    if else_stmts.is_empty() {
                        self.add_edge(cond_id, merge_id, Some("false".to_string()));
                    } else {
                        self.build_statements_with_edge(else_stmts, merge_id, cond_id, Some("false".to_string()));
                    }
                } else {
                    self.add_edge(cond_id, merge_id, Some("false".to_string()));
                }

                if statements.len() > 1 {
                    self.build_statements(&statements[1..], merge_id, exit);
                }
            }
        }
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
                        self.build_statements_with_edge(then_branch, merge_id, cond_id, Some("true".to_string()));
                    }

                    // else分岐
                    if let Some(else_stmts) = else_branch {
                        if else_stmts.is_empty() {
                            self.add_edge(cond_id, merge_id, Some("false".to_string()));
                        } else {
                            self.build_statements_with_edge(else_stmts, merge_id, cond_id, Some("false".to_string()));
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
        // 空のノードを削除し、IDを再割り当て
        let mut id_map = std::collections::HashMap::new();
        let mut new_nodes = Vec::new();
        let mut new_id = 0;

        for node in self.nodes {
            // 空のSimpleノードは削除
            if node.label.is_empty() && node.node_type == NodeType::Simple {
                continue;
            }

            id_map.insert(node.id, new_id);
            new_nodes.push(CFGNode {
                id: new_id,
                node_type: node.node_type,
                label: node.label,
            });
            new_id += 1;
        }

        // 空のノードを通過するエッジをスキップし、IDを再マッピング
        let mut new_edges = Vec::new();
        for edge in self.edges {
            // fromとtoが両方ともマップに存在する場合のみエッジを追加
            if let (Some(&new_from), Some(&new_to)) = (id_map.get(&edge.from), id_map.get(&edge.to)) {
                new_edges.push(CFGEdge {
                    from: new_from,
                    to: new_to,
                    label: edge.label,
                });
            }
        }

        ControlFlowGraph {
            nodes: new_nodes,
            edges: new_edges,
            entry_id: *id_map.get(&entry_id).unwrap_or(&entry_id),
            exit_id: *id_map.get(&exit_id).unwrap_or(&exit_id),
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
